//! BIZ-04: PLU書出しロジック
//!
//! 商品マスタからカシオレジスターツール用PLUファイルを生成し、
//! 保存後に利用者が確認した対象だけ plu_dirty/plu_exported_at を更新する。
//! レジの占有は毎日の SD の読取りで照合する（D-117 (3)）。アプリは枠を消さない（D-117 (1)）。
//!
//! docs/function-design/33-biz-plu-export-service.md に基づく実装。

use crate::db::plu_slot_repo::{self, PluSlot, PluSlotStatus, PluSlotUpdate};
use crate::db::product_repo::{self, Product, ProductForPlu, ProductUpdates};
use crate::db::system_repo::{self, NewOperationLog};
use crate::db::DbConnection;
use crate::io::plu_formatter::{self, PluExportRow, PluFileOutput};
use crate::io::z004_parser;
use sha2::{Digest, Sha256};

use super::BizError;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

const SNAPSHOT_AT_KEY: &str = "plu_register_snapshot_at";
const SNAPSHOT_SETTLEMENT_NO_KEY: &str = "plu_register_snapshot_settlement_no";

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// 書出しモード
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ExportMode {
    /// 全件（is_discontinued=0）
    Full,
    /// 差分（plu_dirty=1）
    Diff,
}

/// PLUファイル生成リクエスト
#[derive(Debug)]
pub struct PluExportPrepareRequest {
    pub mode: ExportMode,
}

/// PLUファイル生成結果
#[derive(Debug)]
pub struct PluExportPreparedResult {
    /// IO-04生成のPLUファイルデータ
    pub plu_output: PluFileOutput,
    /// 書出し件数
    pub count: usize,
    /// PLUファイルに含めた商品コード一覧
    pub target_product_codes: Vec<String>,
    pub prepared_rows: Vec<PluPreparedRow>,
    /// PLUファイルに含めなかった商品一覧
    pub excluded: Vec<PluExcludedProduct>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluExcludedProduct {
    pub product_code: String,
    pub jan_code: Option<String>,
    pub name: String,
    pub memory_no: Option<i64>,
    pub reason: PluExcludedReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluExcludedReason {
    MissingJan,
    InvalidJanFormat,
    InvalidCheckDigit,
    GroupPriceMismatch,
    NoFreeSlot,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PluPreparedRow {
    pub memory_no: i64,
    pub target_product_codes: Vec<String>,
    /// 照合の値: IO-04 が file に書いたこの行の全 11 列の CP932 の byte 列の SHA-256（hex）。
    /// confirm が今の商品から同じ値を求め直して比べる（33 §16.5）
    pub content_hash: String,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct PluRegisterSnapshotSummary {
    pub snapshot_at: Option<String>,
}

/// SD の読取りの後の照合の結果（33 §16.3）
#[derive(Debug, Clone)]
pub enum PluRegisterSnapshotRefresh {
    Refreshed(PluRegisterSnapshotSummary),
    /// 選んだ Z004 の精算回数が前回の照合より小さくて照合しなかった
    SkippedOlder,
    /// ほかの照合しないとき・使える Z004 が無いとき
    Unchanged,
}

/// PLU保存済み確認リクエスト
#[derive(Debug, Clone)]
pub struct PluExportConfirmRequest {
    pub product_codes: Vec<String>,
    pub prepared_rows: Vec<PluPreparedRow>,
}

/// 保存の後に出力の行が変わっていて、未反映のまま残した商品
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct PluRemainingProduct {
    pub product_code: String,
    pub name: String,
}

/// PLU保存済み確認結果
#[derive(Debug)]
pub struct PluExportConfirmResult {
    pub updated_count: usize,
    pub confirmed_at: String,
    pub remaining_products: Vec<PluRemainingProduct>,
}

// ---------------------------------------------------------------------------
// 公開関数
// ---------------------------------------------------------------------------

/// SD の読取りで得た Z004 から、精算回数が最大の 1 本でレジの占有を照合する（33 §16.3 手順 1〜3）
pub fn refresh_plu_register_snapshot_from_sd(
    conn: &mut DbConnection,
    z004_files: &[Vec<u8>],
) -> Result<PluRegisterSnapshotRefresh, BizError> {
    // 手順 1〜2: 精算回数を読めた中の最大の 1 本。同じ値で bytes が違えば照合しない
    let mut best: Option<(i64, &[u8])> = None;
    let mut ambiguous = false;
    for bytes in z004_files {
        let Some(settlement_no) = read_settlement_no(bytes) else {
            continue;
        };
        match best {
            Some((max, _)) if settlement_no < max => {}
            Some((max, kept)) if settlement_no == max => ambiguous |= kept != bytes.as_slice(),
            _ => {
                best = Some((settlement_no, bytes.as_slice()));
                ambiguous = false;
            }
        }
    }
    let Some((settlement_no, bytes)) = best else {
        return Ok(PluRegisterSnapshotRefresh::Unchanged);
    };
    if ambiguous {
        tracing::warn!(
            settlement_no,
            "同じ精算回数で中身の違う Z004 があるので、レジの占有を照合しませんでした"
        );
        return Ok(PluRegisterSnapshotRefresh::Unchanged);
    }
    if let Some(value) = system_repo::get_setting(conn, SNAPSHOT_SETTLEMENT_NO_KEY)? {
        match parse_settlement_no(&value) {
            None => {
                tracing::warn!(
                    "前回の照合の精算回数を読めないので、レジの占有を照合しませんでした"
                );
                return Ok(PluRegisterSnapshotRefresh::Unchanged);
            }
            Some(previous) if settlement_no < previous => {
                tracing::warn!(
                    settlement_no,
                    previous,
                    "SD の Z004 の精算回数が前回の照合より小さいので、レジの占有を照合しませんでした"
                );
                return Ok(PluRegisterSnapshotRefresh::SkippedOlder);
            }
            // 既に写した観測。confirm したばかりの枠を、書込みの前の Z004 で未反映へ戻さない
            Some(previous) if settlement_no == previous => {
                return Ok(PluRegisterSnapshotRefresh::Unchanged)
            }
            Some(_) => {}
        }
    }
    import_plu_register_snapshot(conn, bytes, settlement_no)
        .map(PluRegisterSnapshotRefresh::Refreshed)
}

/// 照合の本体（33 §16.3 の表）。1 つの TX で枠・app_settings・操作ログを書く
fn import_plu_register_snapshot(
    conn: &mut DbConnection,
    raw_bytes: &[u8],
    settlement_no: i64,
) -> Result<PluRegisterSnapshotSummary, BizError> {
    let observed = z004_parser::parse_plu_register_snapshot(raw_bytes)
        .map_err(|error| BizError::ImportError(error.to_string()))?;
    let snapshot_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let tx = conn.transaction().map_err(db_tx_error)?;

    // 判定は照合の前の状態で行う（同じ照合の中で先に変えた枠の結果を、後の枠の判定に使わない）
    let before: HashMap<i64, PluSlot> = plu_slot_repo::list_slots(&tx)?
        .into_iter()
        .map(|slot| (slot.memory_no, slot))
        .collect();
    let managed_jans: HashSet<&str> = before
        .values()
        .filter(|slot| matches!(slot.status, PluSlotStatus::Reserved | PluSlotStatus::Active))
        .filter_map(|slot| slot.scanning_code.as_deref())
        .collect();
    // 同じ Z004 の中で同じ JAN を複数の空き枠で観測したら、最小の memory No. だけを採用する（観測は memory No. の昇順）
    let mut adopted: HashSet<String> = HashSet::new();
    let (mut conflict, mut missing, mut dropped, mut duplicate, mut released) = (0, 0, 0, 0, 0);
    let set_slot = |memory_no: i64, code: Option<&str>, status: PluSlotStatus, kept: &PluSlot| {
        let active = status == PluSlotStatus::Active;
        plu_slot_repo::update_slot(
            &tx,
            PluSlotUpdate {
                memory_no,
                scanning_code: code,
                status,
                reserved_at: active.then_some(kept.reserved_at.as_deref()).flatten(),
                activated_at: active.then_some(snapshot_at.as_str()),
                updated_at: &snapshot_at,
            },
        )
    };

    for observed_slot in observed.into_iter().filter(|slot| slot.memory_no >= 217) {
        let current = before.get(&observed_slot.memory_no).ok_or_else(|| {
            BizError::ImportError(format!(
                "PLU slot {} が存在しません",
                observed_slot.memory_no
            ))
        })?;
        let no = current.memory_no;
        let current_code = current.scanning_code.as_deref();
        match (observed_slot.raw_code.as_deref(), current.status) {
            (None, PluSlotStatus::Free | PluSlotStatus::Reserved) => {}
            (None, PluSlotStatus::External) => set_slot(no, None, PluSlotStatus::Free, current)?,
            (None, PluSlotStatus::Active) => {
                missing += 1;
                mark_target_products_dirty(&tx, current_code)?;
            }
            (Some(code), PluSlotStatus::Free) => {
                let eligible =
                    validate_plu_jan(Some(code)).is_ok() && has_target_product(&tx, code)?;
                if !eligible {
                    set_slot(no, Some(code), PluSlotStatus::External, current)?;
                } else if managed_jans.contains(code) || !adopted.insert(code.to_string()) {
                    duplicate += 1;
                    set_slot(no, Some(code), PluSlotStatus::External, current)?;
                } else {
                    set_slot(no, Some(code), PluSlotStatus::Active, current)?;
                }
            }
            (Some(code), PluSlotStatus::Reserved) if current_code == Some(code) => {
                set_slot(no, Some(code), PluSlotStatus::Active, current)?;
            }
            (Some(code), PluSlotStatus::Reserved) => {
                dropped += 1;
                set_slot(no, Some(code), PluSlotStatus::External, current)?;
            }
            (Some(code), PluSlotStatus::Active | PluSlotStatus::External)
                if current_code == Some(code) => {}
            (Some(code), PluSlotStatus::Active) => {
                // アプリが書く商品が無い JAN の枠は、レジ側の登録を書き直さない
                if current_code.map_or(Ok(false), |jan| has_target_product(&tx, jan))? {
                    conflict += 1;
                    mark_target_products_dirty(&tx, current_code)?;
                } else {
                    released += 1;
                    set_slot(no, Some(code), PluSlotStatus::External, current)?;
                }
            }
            (Some(code), PluSlotStatus::External) => {
                set_slot(no, Some(code), PluSlotStatus::External, current)?;
            }
        }
    }

    let (free_count, external_count, app_managed_count): (i64, i64, i64) = tx
        .query_row(
            "SELECT SUM(status='free'), SUM(status='external'), SUM(status IN ('reserved','active'))
             FROM plu_slots",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(crate::db::DbError::from)?;
    system_repo::upsert_setting(&tx, SNAPSHOT_AT_KEY, &snapshot_at)?;
    system_repo::upsert_setting(&tx, SNAPSHOT_SETTLEMENT_NO_KEY, &settlement_no.to_string())?;
    // 件数だけを残す（実コード・名称・単価は入れない）
    system_repo::insert_operation_log(
        &tx,
        &NewOperationLog {
            operation_type: "plu_register_snapshot_import".to_string(),
            summary: format!("レジ登録状況を読み込みました（conflict {conflict}件）"),
            detail_json: Some(
                serde_json::json!({
                    "free_count": free_count,
                    "external_count": external_count,
                    "app_managed_count": app_managed_count,
                    "conflict_count": conflict,
                    "missing_count": missing,
                    "dropped_count": dropped,
                    "duplicate_count": duplicate,
                    "released_count": released,
                })
                .to_string(),
            ),
        },
    )?;
    tx.commit().map_err(db_tx_error)?;
    Ok(PluRegisterSnapshotSummary {
        snapshot_at: Some(snapshot_at),
    })
}

pub fn get_plu_slot_summary(conn: &DbConnection) -> Result<PluRegisterSnapshotSummary, BizError> {
    Ok(PluRegisterSnapshotSummary {
        snapshot_at: system_repo::get_setting(conn, SNAPSHOT_AT_KEY)?,
    })
}

/// 指定モードでPLUファイルを生成し、slot予約を永続化する。
///
/// 33-biz-plu-export-service.md セクション16.4
pub fn prepare_plu_export(
    conn: &mut DbConnection,
    req: PluExportPrepareRequest,
) -> Result<PluExportPreparedResult, BizError> {
    if system_repo::get_setting(conn, SNAPSHOT_AT_KEY)?.is_none() {
        return Err(BizError::ValidationFailed(
            "register_snapshot_required: レジの登録状況がまだ読まれていません。売上データ取込みでレジの SD を一度読んでください".to_string(),
        ));
    }
    let tx = conn.transaction().map_err(db_tx_error)?;
    // 群は、その JAN の対象の商品の全員で作る（Diff も。未反映の商品だけで比べると、共有 JAN の片方の変更を見逃す）
    let targets = product_repo::find_active_products_for_plu(&tx)?;
    let selected =
        |product: &ProductForPlu| req.mode == ExportMode::Full || product.product.plu_dirty;

    let mut excluded = Vec::new();
    let mut valid_groups: BTreeMap<String, Vec<&ProductForPlu>> = BTreeMap::new();
    for product in &targets {
        match validate_plu_jan(product.product.jan_code.as_deref()) {
            Ok(jan) => valid_groups
                .entry(jan.to_string())
                .or_default()
                .push(product),
            Err(reason) if selected(product) => excluded.push(excluded_product(product, reason)),
            Err(_) => {}
        }
    }

    let mut rows = Vec::new();
    let mut prepared_rows = Vec::new();
    let mut target_product_codes = Vec::new();
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    for (jan, group) in valid_groups {
        if !group.iter().any(|product| selected(product)) {
            continue;
        }
        let Some(first) = group_representative(&group) else {
            excluded.extend(
                group
                    .iter()
                    .map(|p| excluded_product(p, PluExcludedReason::GroupPriceMismatch)),
            );
            continue;
        };

        let slots = plu_slot_repo::find_slots_by_scanning_code(&tx, &jan)?;
        let reserve = |memory_no: i64, status: PluSlotStatus| {
            plu_slot_repo::update_slot(
                &tx,
                PluSlotUpdate {
                    memory_no,
                    scanning_code: Some(&jan),
                    status,
                    reserved_at: (status == PluSlotStatus::Reserved).then_some(now.as_str()),
                    activated_at: (status == PluSlotStatus::Active).then_some(now.as_str()),
                    updated_at: &now,
                },
            )
        };
        // sticky → 同じコードの external の採用（最小の memory No.）→ 最小の空き
        let memory_no = if let Some(slot) = slots
            .iter()
            .find(|slot| matches!(slot.status, PluSlotStatus::Reserved | PluSlotStatus::Active))
        {
            slot.memory_no
        } else if let Some(slot) = slots
            .iter()
            .find(|slot| slot.status == PluSlotStatus::External)
        {
            reserve(slot.memory_no, PluSlotStatus::Active)?;
            slot.memory_no
        } else if let Some(slot) = plu_slot_repo::find_min_free_slot(&tx)? {
            reserve(slot.memory_no, PluSlotStatus::Reserved)?;
            slot.memory_no
        } else {
            excluded.extend(
                group
                    .iter()
                    .map(|p| excluded_product(p, PluExcludedReason::NoFreeSlot)),
            );
            continue;
        };

        let row = export_row(memory_no, first);
        let group_codes: Vec<String> = group
            .iter()
            .map(|p| p.product.product_code.clone())
            .collect();
        target_product_codes.extend(group_codes.iter().cloned());
        prepared_rows.push(PluPreparedRow {
            memory_no,
            target_product_codes: group_codes,
            content_hash: row_content_hash(&row)?,
        });
        rows.push(row);
    }
    excluded.sort_by(|a, b| a.product_code.cmp(&b.product_code));

    rows.sort_by_key(|row| row.memory_no);
    prepared_rows.sort_by_key(|row| row.memory_no);
    let count = rows.len();

    if rows.is_empty() {
        return Err(BizError::ValidationFailed(build_all_excluded_message(
            &excluded,
        )));
    }

    let plu_output = plu_formatter::generate_plu_tsv(&rows).map_err(format_error)?;

    tx.commit().map_err(db_tx_error)?;

    Ok(PluExportPreparedResult {
        plu_output,
        count,
        target_product_codes,
        prepared_rows,
        excluded,
    })
}

/// 保存済み確認された商品のうち、出力の行が今も同じ分だけPLU未反映状態を解除する。
///
/// 33-biz-plu-export-service.md セクション16.5
pub fn confirm_plu_export_saved(
    conn: &mut DbConnection,
    req: PluExportConfirmRequest,
) -> Result<PluExportConfirmResult, BizError> {
    if req.product_codes.is_empty() {
        return Err(BizError::ValidationFailed(
            "書出し済みにする商品がありません".to_string(),
        ));
    }
    if req.prepared_rows.is_empty() {
        return Err(BizError::ValidationFailed(
            "prepared_rows が空です".to_string(),
        ));
    }
    let mut seen = HashSet::new();
    for code in &req.product_codes {
        if code.trim().is_empty() {
            return Err(BizError::ValidationFailed("商品コードが空です".to_string()));
        }
        if !seen.insert(code) {
            return Err(BizError::ValidationFailed(format!(
                "同じ商品コードが複数含まれています: {}",
                code
            )));
        }
    }

    let confirmed_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let mut updated_count = 0;
    let mut remaining_products = Vec::new();
    {
        let tx = conn.transaction().map_err(db_tx_error)?;

        let prepared_codes: BTreeSet<_> = req
            .prepared_rows
            .iter()
            .flat_map(|row| row.target_product_codes.iter().cloned())
            .collect();
        let request_codes: BTreeSet<_> = req.product_codes.iter().cloned().collect();
        if prepared_codes != request_codes {
            return Err(BizError::ValidationFailed(
                "prepare と confirm の商品集合が一致しません".to_string(),
            ));
        }
        // 今の商品から行を求め直すための、JAN ごとの対象の商品の全員（prepare と同じ出どころ）
        let targets = product_repo::find_active_products_for_plu(&tx)?;
        let mut memory_numbers = HashSet::new();
        for row in &req.prepared_rows {
            if !memory_numbers.insert(row.memory_no) {
                return Err(BizError::ValidationFailed(
                    "同じPLUメモリNo.が複数含まれています".to_string(),
                ));
            }
            let slot =
                plu_slot_repo::find_slot_by_memory_no(&tx, row.memory_no)?.ok_or_else(|| {
                    BizError::ValidationFailed(format!("PLU slot {} が存在しません", row.memory_no))
                })?;
            if row.target_product_codes.is_empty() {
                return Err(BizError::ValidationFailed(
                    "product行の商品集合が空です".to_string(),
                ));
            }
            if !matches!(slot.status, PluSlotStatus::Reserved | PluSlotStatus::Active) {
                return Err(BizError::ValidationFailed(
                    "PLU slot状態がprepare時と一致しません".to_string(),
                ));
            }
            let mut row_products = Vec::with_capacity(row.target_product_codes.len());
            for product_code in &row.target_product_codes {
                let product =
                    product_repo::find_by_product_code(&tx, product_code)?.ok_or_else(|| {
                        BizError::NotFound(format!("商品 {} が見つかりません", product_code))
                    })?;
                if product.product.jan_code.as_deref() != slot.scanning_code.as_deref() {
                    return Err(BizError::ValidationFailed(
                        "prepare と confirm のJAN/slot対応が一致しません".to_string(),
                    ));
                }
                row_products.push(product.product);
            }

            // 保存の後に出力の行が変わった商品は、未反映のまま残す（書き出した file は古い内容）
            let group: Vec<&ProductForPlu> = targets
                .iter()
                .filter(|p| p.product.jan_code.as_deref() == slot.scanning_code.as_deref())
                .collect();
            let current_hash = match group_representative(&group) {
                Some(first) => Some(row_content_hash(&export_row(slot.memory_no, first))?),
                None => None,
            };
            if current_hash.as_deref() != Some(row.content_hash.as_str()) {
                remaining_products.extend(row_products.into_iter().map(|product| {
                    PluRemainingProduct {
                        product_code: product.product_code,
                        name: product.name,
                    }
                }));
                continue;
            }

            if slot.status == PluSlotStatus::Reserved {
                plu_slot_repo::update_slot(
                    &tx,
                    PluSlotUpdate {
                        memory_no: slot.memory_no,
                        scanning_code: slot.scanning_code.as_deref(),
                        status: PluSlotStatus::Active,
                        reserved_at: slot.reserved_at.as_deref(),
                        activated_at: Some(&confirmed_at),
                        updated_at: &confirmed_at,
                    },
                )?;
            }
            let updates = ProductUpdates {
                plu_dirty: Some(false),
                plu_exported_at: Some(Some(confirmed_at.clone())),
                ..Default::default()
            };
            for product_code in &row.target_product_codes {
                if !product_repo::update_product(&tx, product_code, &updates)? {
                    return Err(BizError::NotFound(format!(
                        "商品 {} が見つかりません",
                        product_code
                    )));
                }
                updated_count += 1;
            }
        }

        system_repo::insert_operation_log(
            &tx,
            &NewOperationLog {
                operation_type: "plu_export".to_string(),
                summary: format!("PLU書出し済み確認を記録しました（{updated_count}件）"),
                detail_json: Some(
                    serde_json::json!({
                        "count": updated_count,
                        "product_count": updated_count,
                        "remaining_count": remaining_products.len(),
                        "confirmed_at": confirmed_at,
                    })
                    .to_string(),
                ),
            },
        )?;
        tx.commit().map_err(db_tx_error)?;
    }

    Ok(PluExportConfirmResult {
        updated_count,
        confirmed_at,
        remaining_products,
    })
}

/// plu_dirty=1の商品一覧を返す（UI-08の差分対象プレビュー用）
///
/// 33-biz-plu-export-service.md セクション16.6.1
pub fn list_plu_dirty(conn: &DbConnection) -> Result<Vec<Product>, BizError> {
    Ok(product_repo::find_plu_dirty_products(conn)?)
}

fn validate_plu_jan(jan_code: Option<&str>) -> Result<&str, PluExcludedReason> {
    let Some(jan) = jan_code else {
        return Err(PluExcludedReason::MissingJan);
    };
    if jan.len() != 13 || !jan.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(PluExcludedReason::InvalidJanFormat);
    }
    if !plu_formatter::is_valid_ean13_code(jan) {
        return Err(PluExcludedReason::InvalidCheckDigit);
    }
    Ok(jan)
}

fn excluded_product(product: &ProductForPlu, reason: PluExcludedReason) -> PluExcludedProduct {
    PluExcludedProduct {
        product_code: product.product.product_code.clone(),
        jan_code: product.product.jan_code.clone(),
        name: product.product.name.clone(),
        memory_no: None,
        reason,
    }
}

/// 同じ JAN の対象の商品の群の代表（商品コードの最小。D-028）。売価か税率がそろわなければ None
fn group_representative<'a>(group: &[&'a ProductForPlu]) -> Option<&'a ProductForPlu> {
    let first = *group
        .iter()
        .min_by(|a, b| a.product.product_code.cmp(&b.product.product_code))?;
    group
        .iter()
        .all(|p| {
            p.product.selling_price == first.product.selling_price
                && p.product.tax_rate == first.product.tax_rate
        })
        .then_some(first)
}

fn export_row(memory_no: i64, product: &ProductForPlu) -> PluExportRow {
    PluExportRow {
        memory_no,
        product_code: product.product.product_code.clone(),
        jan_code: product.product.jan_code.clone(),
        name: product.product.name.clone(),
        selling_price: product.product.selling_price,
        tax_rate: product.product.tax_rate.as_str().to_string(),
        department_name: product.department_name.clone(),
    }
}

/// 照合の値。入力は IO-04 が file に書く 1 行の byte 列だけ（切り詰めで落ちる名称の末尾は入らない）
fn row_content_hash(row: &PluExportRow) -> Result<String, BizError> {
    let bytes = plu_formatter::format_plu_row(row).map_err(format_error)?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

fn format_error(error: plu_formatter::PluFormatError) -> BizError {
    BizError::ExportError(format!("PLUファイルの生成に失敗しました: {}", error))
}

/// 精算回数（IO-02 の精算の識別メタ）。読めない bytes・読めない値は None
fn read_settlement_no(bytes: &[u8]) -> Option<i64> {
    let metadata = z004_parser::parse_z004(bytes).ok()?.settlement_metadata?;
    parse_settlement_no(&metadata.settlement_no?)
}

/// 10 進の整数（ASCII の数字だけ。前の 0 は許す）
fn parse_settlement_no(value: &str) -> Option<i64> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

fn build_all_excluded_message(excluded: &[PluExcludedProduct]) -> String {
    let details = excluded
        .iter()
        .map(|p| format!("{}（{}）", p.product_code, excluded_reason_label(&p.reason)))
        .collect::<Vec<_>>()
        .join("、");
    format!(
        "PLUファイルに書き出せる商品がありません。商品マスタで13桁JANを確認してください。対象: {}",
        details
    )
}

fn excluded_reason_label(reason: &PluExcludedReason) -> &'static str {
    match reason {
        PluExcludedReason::MissingJan => "JAN未登録",
        PluExcludedReason::InvalidJanFormat => "JANが13桁ではありません",
        PluExcludedReason::InvalidCheckDigit => "JANのチェックディジットが不正です",
        PluExcludedReason::GroupPriceMismatch => "同じJANの商品で売価または税率が一致していません",
        PluExcludedReason::NoFreeSlot => "レジの空きスロットがありません",
    }
}

fn db_tx_error(error: rusqlite::Error) -> BizError {
    BizError::DatabaseError(crate::db::DbError::QueryFailed(error.to_string()))
}

/// その JAN に対象の商品（`plu_target=1`・未廃番）があるか
fn has_target_product(conn: &DbConnection, jan_code: &str) -> Result<bool, BizError> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM products WHERE jan_code=?1 AND plu_target=1 AND is_discontinued=0)",
        rusqlite::params![jan_code],
        |row| row.get(0),
    ).map_err(crate::db::DbError::from)?)
}

fn mark_target_products_dirty(conn: &DbConnection, jan_code: Option<&str>) -> Result<(), BizError> {
    if let Some(jan_code) = jan_code {
        conn.execute(
            "UPDATE products SET plu_dirty=1 WHERE jan_code=?1 AND plu_target=1",
            rusqlite::params![jan_code],
        )
        .map_err(crate::db::DbError::from)?;
    }
    Ok(())
}

// ===========================================================================
// テスト
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::product_repo::{self, NewProduct};
    use crate::db::test_support::setup_test_db;

    fn valid_jan_from_seed(seed: u64) -> String {
        let body = format!("{:012}", 490_000_000_000_u64 + seed);
        let sum: u32 = body
            .chars()
            .enumerate()
            .map(|(idx, ch)| {
                let digit = ch.to_digit(10).unwrap();
                if idx % 2 == 0 {
                    digit
                } else {
                    digit * 3
                }
            })
            .sum();
        let check = (10 - (sum % 10)) % 10;
        format!("{}{}", body, check)
    }

    fn jan_seed_from_product_code(product_code: &str) -> u64 {
        product_code
            .bytes()
            .fold(0_u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
            % 900_000
    }

    /// テスト用に商品を登録するヘルパー
    fn seed_product_for_plu(
        conn: &DbConnection,
        product_code: &str,
        name: &str,
        department_id: i64,
        is_discontinued: bool,
        plu_dirty: bool,
    ) {
        let jan_code = valid_jan_from_seed(jan_seed_from_product_code(product_code));
        seed_product_for_plu_with_jan(
            conn,
            product_code,
            Some(&jan_code),
            name,
            department_id,
            is_discontinued,
            plu_dirty,
        );
    }

    fn seed_product_for_plu_with_jan(
        conn: &DbConnection,
        product_code: &str,
        jan_code: Option<&str>,
        name: &str,
        department_id: i64,
        is_discontinued: bool,
        plu_dirty: bool,
    ) {
        let product = NewProduct {
            product_code: product_code.to_string(),
            jan_code: jan_code.map(String::from),
            name: name.to_string(),
            department_id,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity: 10,
            stock_unit: "pcs".to_string(),
            is_discontinued,
            plu_dirty,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(conn, &product).unwrap();
        system_repo::upsert_setting(conn, "plu_register_snapshot_at", "2026-08-18T00:00:00")
            .unwrap();
    }

    /// 今の商品から求めた行の照合の値（test の fixture 用。無い商品は一致しない値）
    fn current_hash(conn: &DbConnection, memory_no: i64, product_code: &str) -> String {
        product_repo::find_active_products_for_plu(conn)
            .unwrap()
            .iter()
            .find(|product| product.product.product_code == product_code)
            .map(|product| row_content_hash(&export_row(memory_no, product)).unwrap())
            .unwrap_or_else(|| "no-such-product".to_string())
    }

    fn prepared_rows_for_codes(conn: &DbConnection, codes: &[String]) -> Vec<PluPreparedRow> {
        let now = "2026-08-18T00:00:00";
        let mut rows = Vec::new();
        for (index, code) in codes
            .iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .enumerate()
        {
            let memory_no = 217 + index as i64;
            let scanning_code = product_repo::find_by_product_code(conn, code)
                .unwrap()
                .and_then(|product| product.product.jan_code)
                .unwrap_or_else(|| format!("SYNTH-{index}"));
            plu_slot_repo::update_slot(
                conn,
                PluSlotUpdate {
                    memory_no,
                    scanning_code: Some(&scanning_code),
                    status: PluSlotStatus::Reserved,
                    reserved_at: Some(now),
                    activated_at: None,
                    updated_at: now,
                },
            )
            .unwrap();
            rows.push(PluPreparedRow {
                memory_no,
                target_product_codes: vec![(*code).clone()],
                content_hash: current_hash(conn, memory_no, code),
            });
        }
        rows
    }

    /// 合成の全枠の Z004（layout A、CP932・CRLF、5,000 行）。`codes` に無い枠は空
    fn snapshot_bytes(settlement_no: &str, codes: &[(i64, String)]) -> Vec<u8> {
        let by_memory: std::collections::HashMap<i64, &str> = codes
            .iter()
            .map(|(memory_no, code)| (*memory_no, code.as_str()))
            .collect();
        let mut text = format!(
            "\"マシンNo.   \",\"0001\"\r\n\"ファイル    \",\"Z004_SYNTH\"\r\n\"精算回数    \",\"{settlement_no}\"\r\n\"日付        \",\"2026-08-15\"\r\n\"時刻        \",\"18:30\"\r\n\r\n\"メモリNo.\",\"ｽｷｬﾆﾝｸﾞｺｰﾄﾞ\",\"名称\",\"個数\",\"金額\"\r\n"
        );
        for memory_no in 1..=5_000 {
            let code = by_memory
                .get(&(memory_no as i64))
                .copied()
                .unwrap_or("00000000000000");
            text.push_str(&format!("{memory_no},{code},,0,0\r\n"));
        }
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode(&text);
        bytes.into_owned()
    }

    fn put_slot(conn: &DbConnection, memory_no: i64, code: Option<&str>, status: PluSlotStatus) {
        let activated = (status == PluSlotStatus::Active).then_some("2026-08-17T00:00:00");
        let reserved = (status == PluSlotStatus::Reserved).then_some("2026-08-17T00:00:00");
        plu_slot_repo::update_slot(
            conn,
            PluSlotUpdate {
                memory_no,
                scanning_code: code,
                status,
                reserved_at: reserved,
                activated_at: activated,
                updated_at: "2026-08-17T00:00:00",
            },
        )
        .unwrap();
    }

    fn slot(conn: &DbConnection, memory_no: i64) -> PluSlot {
        plu_slot_repo::find_slot_by_memory_no(conn, memory_no)
            .unwrap()
            .unwrap()
    }

    fn prepare(conn: &mut DbConnection, mode: ExportMode) -> PluExportPreparedResult {
        prepare_plu_export(conn, PluExportPrepareRequest { mode }).unwrap()
    }

    fn memory_numbers(result: &PluExportPreparedResult) -> Vec<i64> {
        result
            .prepared_rows
            .iter()
            .map(|row| row.memory_no)
            .collect()
    }

    fn set_plu_state(conn: &DbConnection, product_code: &str, plu_target: bool, plu_dirty: bool) {
        conn.execute(
            "UPDATE products SET plu_target=?2, plu_dirty=?3 WHERE product_code=?1",
            rusqlite::params![product_code, plu_target, plu_dirty],
        )
        .unwrap();
    }

    fn plu_dirty(conn: &DbConnection, product_code: &str) -> bool {
        product_repo::find_by_product_code(conn, product_code)
            .unwrap()
            .unwrap()
            .product
            .plu_dirty
    }

    /// 枠・app_settings・操作ログの全行（照合しないときに何も変わらないことの比較用）
    fn reconcile_state(conn: &DbConnection) -> Vec<Vec<Vec<rusqlite::types::Value>>> {
        use crate::db::test_support::dump;
        vec![
            dump(conn, "SELECT * FROM plu_slots ORDER BY memory_no"),
            dump(conn, "SELECT * FROM app_settings ORDER BY key"),
            dump(conn, "SELECT * FROM operation_logs ORDER BY id"),
            dump(
                conn,
                "SELECT product_code, plu_dirty FROM products ORDER BY 1",
            ),
        ]
    }

    #[test]
    fn test_prepare_plu_export_req907_requires_snapshot_without_writes() {
        // REQ-907 / 33 §16.3 の gate: 一度も照合していなければ、空きを仮定せずに拒む
        let (_dir, mut conn) = setup_test_db();
        assert_eq!(get_plu_slot_summary(&conn).unwrap().snapshot_at, None);
        let before = reconcile_state(&conn);
        let result = prepare_plu_export(
            &mut conn,
            PluExportPrepareRequest {
                mode: ExportMode::Full,
            },
        );
        assert!(
            matches!(result, Err(BizError::ValidationFailed(ref message)) if message == "register_snapshot_required: レジの登録状況がまだ読まれていません。売上データ取込みでレジの SD を一度読んでください"),
            "{result:?}"
        );
        assert_eq!(reconcile_state(&conn), before);
    }

    #[test]
    fn test_import_plu_register_snapshot_req907_reconciliation_table() {
        // REQ-907 / BIZ-04-D3: 33 §16.3 の表の行ごと。判定は照合の前の状態で行う
        let (_dir, mut conn) = setup_test_db();
        let jan = |seed| valid_jan_from_seed(seed);
        let seed = |code: &str, jan: &str, plu_target: bool| {
            seed_product_for_plu_with_jan(&conn, code, Some(jan), code, 1, false, false);
            set_plu_state(&conn, code, plu_target, false);
        };
        let (eligible, managed_active, managed_reserved) = (jan(81), jan(86), jan(87));
        let (missing, conflict, reserved_same, dropped) = (jan(82), jan(83), jan(84), jan(85));
        let (untargeted, no_product, off_target, dropped_then_seen) =
            (jan(88), jan(89), jan(90), jan(91));
        seed("SNAP-ELIG", &eligible, true);
        seed("SNAP-MANAGED-A", &managed_active, true);
        seed("SNAP-MANAGED-R", &managed_reserved, true);
        seed("SNAP-MISS", &missing, true);
        seed("SNAP-MISS-OFF", &missing, false);
        seed("SNAP-CONFLICT", &conflict, true);
        seed("SNAP-UNTARGETED", &untargeted, false);
        seed("SNAP-OFF", &off_target, false);
        seed("SNAP-PRE", &dropped_then_seen, true);
        system_repo::delete_setting(&conn, SNAPSHOT_AT_KEY).unwrap();

        put_slot(&conn, 219, Some("RES-EMPTY"), PluSlotStatus::Reserved);
        put_slot(&conn, 220, Some(&reserved_same), PluSlotStatus::Reserved);
        put_slot(&conn, 221, Some(&missing), PluSlotStatus::Active);
        put_slot(&conn, 222, Some(&conflict), PluSlotStatus::Active);
        put_slot(&conn, 223, Some("EXT-EMPTY"), PluSlotStatus::External);
        put_slot(&conn, 224, Some("EXT-SAME"), PluSlotStatus::External);
        put_slot(&conn, 225, Some("EXT-OLD"), PluSlotStatus::External);
        put_slot(&conn, 229, Some(&dropped), PluSlotStatus::Reserved);
        put_slot(&conn, 231, Some(&managed_active), PluSlotStatus::Active);
        put_slot(&conn, 233, Some(&managed_reserved), PluSlotStatus::Reserved);
        put_slot(&conn, 235, Some(&untargeted), PluSlotStatus::Active);
        put_slot(&conn, 236, Some(&no_product), PluSlotStatus::Active);
        put_slot(
            &conn,
            239,
            Some(&dropped_then_seen),
            PluSlotStatus::Reserved,
        );
        let untouched_free = slot(&conn, 238);

        let summary = import_plu_register_snapshot(
            &mut conn,
            &snapshot_bytes(
                "0042",
                &[
                    (217, eligible.clone()),
                    (218, eligible.clone()),
                    (220, reserved_same.clone()),
                    (222, "DIFFERENT-A".to_string()),
                    (224, "EXT-SAME".to_string()),
                    (225, "EXT-NEW".to_string()),
                    (229, "DIFFERENT-R".to_string()),
                    (230, "12345678EEEEEE".to_string()),
                    (231, managed_active.clone()),
                    (232, managed_active.clone()),
                    (234, managed_reserved.clone()),
                    (235, "DIFFERENT-U".to_string()),
                    (236, "DIFFERENT-N".to_string()),
                    (237, off_target.clone()),
                    (239, "DIFFERENT-P".to_string()),
                    (240, dropped_then_seen.clone()),
                ],
            ),
            42,
        )
        .unwrap();

        let state = |memory_no| {
            let slot = slot(&conn, memory_no);
            (slot.status, slot.scanning_code)
        };
        let code = |value: &str| Some(value.to_string());
        use PluSlotStatus::{Active, External, Free, Reserved};
        // occupied × free: 対象の JAN は最小の番号だけ採用し、残りは external（観測コードを保持）
        assert_eq!(state(217), (Active, code(&eligible)));
        assert!(slot(&conn, 217).activated_at.is_some());
        assert_eq!(state(218), (External, code(&eligible)));
        // 空 × reserved は維持、同一コード × reserved は active（reserved_at は保つ）
        assert_eq!(state(219), (Reserved, code("RES-EMPTY")));
        assert_eq!(state(220), (Active, code(&reserved_same)));
        assert_eq!(
            slot(&conn, 220).reserved_at.as_deref(),
            Some("2026-08-17T00:00:00")
        );
        // 空 × active は維持して missing、別コード × active（対象の商品あり）は維持して conflict
        assert_eq!(state(221), (Active, code(&missing)));
        assert_eq!(state(222), (Active, code(&conflict)));
        // 空 × external は free、同一コードは維持、別コードは観測値で更新
        assert_eq!(state(223), (Free, None));
        assert_eq!(state(224), (External, code("EXT-SAME")));
        assert_eq!(state(225), (External, code("EXT-NEW")));
        // 別コード × reserved は external（reservation_dropped）
        assert_eq!(state(229), (External, code("DIFFERENT-R")));
        // occupied × free: 対象の JAN でないコードは external
        assert_eq!(state(230), (External, code("12345678EEEEEE")));
        assert_eq!(state(237), (External, code(&off_target)));
        // アプリの枠を持つ JAN の重複の観測は external。アプリの枠は変えない
        assert_eq!(state(231), (Active, code(&managed_active)));
        assert_eq!(state(232), (External, code(&managed_active)));
        assert_eq!(state(233), (Reserved, code(&managed_reserved)));
        assert_eq!(state(234), (External, code(&managed_reserved)));
        // 別コード × active（対象の商品が無い JAN）は external
        assert_eq!(state(235), (External, code("DIFFERENT-U")));
        assert_eq!(state(236), (External, code("DIFFERENT-N")));
        // 照合の前の状態で判定: 239 の予約が同じ照合で落ちても、240 の観測は「アプリの枠を持つ JAN」の重複
        assert_eq!(state(239), (External, code("DIFFERENT-P")));
        assert_eq!(state(240), (External, code(&dropped_then_seen)));
        // 空 × free は何も書かない
        assert_eq!(slot(&conn, 238), untouched_free);

        // dirty にするのは plu_target=1 の商品だけ
        for (product_code, expected) in [
            ("SNAP-MISS", true),
            ("SNAP-MISS-OFF", false),
            ("SNAP-CONFLICT", true),
            ("SNAP-UNTARGETED", false),
            ("SNAP-ELIG", false),
        ] {
            assert_eq!(plu_dirty(&conn, product_code), expected, "{product_code}");
        }

        assert_eq!(
            summary.snapshot_at,
            system_repo::get_setting(&conn, SNAPSHOT_AT_KEY).unwrap()
        );
        assert!(summary.snapshot_at.is_some());
        assert_eq!(
            get_plu_slot_summary(&conn).unwrap().snapshot_at,
            summary.snapshot_at
        );
        assert_eq!(
            system_repo::get_setting(&conn, SNAPSHOT_SETTLEMENT_NO_KEY)
                .unwrap()
                .as_deref(),
            Some("42")
        );
        let detail: String = conn
            .query_row(
                "SELECT detail_json FROM operation_logs WHERE operation_type='plu_register_snapshot_import'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let detail: serde_json::Value = serde_json::from_str(&detail).unwrap();
        assert_eq!(
            detail,
            serde_json::json!({
                "free_count": 4_784 - 19,
                "external_count": 12,
                "app_managed_count": 7,
                "conflict_count": 1,
                "missing_count": 1,
                "dropped_count": 2,
                "duplicate_count": 4,
                "released_count": 2,
            })
        );
    }

    #[test]
    fn test_import_plu_register_snapshot_req907_rolls_back_settings_log_and_slots() {
        // REQ-907: 照合の枠・key・操作ログは 1 つの TX。途中の失敗で何も残らない
        let (_dir, mut conn) = setup_test_db();
        system_repo::upsert_setting(&conn, SNAPSHOT_AT_KEY, "before-at").unwrap();
        system_repo::upsert_setting(&conn, SNAPSHOT_SETTLEMENT_NO_KEY, "7").unwrap();
        put_slot(&conn, 218, Some("EXT-EMPTY"), PluSlotStatus::External);
        // 操作ログの INSERT（TX の最後の書込み）を失敗させる
        conn.execute_batch(
            "CREATE TRIGGER fail_snapshot_log BEFORE INSERT ON operation_logs
             WHEN NEW.operation_type = 'plu_register_snapshot_import'
             BEGIN SELECT RAISE(ABORT, 'injected'); END;",
        )
        .unwrap();
        let before = reconcile_state(&conn);

        let result = import_plu_register_snapshot(
            &mut conn,
            &snapshot_bytes("0008", &[(217, "12345678EEEEEE".to_string())]),
            8,
        );

        assert!(matches!(result, Err(BizError::DatabaseError(_))));
        assert_eq!(reconcile_state(&conn), before);
    }

    #[test]
    fn test_refresh_plu_register_snapshot_from_sd_req907_picks_max_settlement_no() {
        // REQ-907 / 33 §16.3 手順 1〜2: 精算回数が最大の 1 本だけを照合する
        let (_dir, mut conn) = setup_test_db();
        let at = |memory_no: i64, code: &str| vec![(memory_no, code.to_string())];
        let files = vec![
            snapshot_bytes("0003", &at(300, "OLD-3")),
            b"not a z004".to_vec(),
            snapshot_bytes("0010", &at(301, "NEW-10")),
            snapshot_bytes("x1", &at(302, "BAD-NO")),
            snapshot_bytes("0007", &at(303, "OLD-7")),
        ];

        let refresh = refresh_plu_register_snapshot_from_sd(&mut conn, &files).unwrap();

        assert!(matches!(refresh, PluRegisterSnapshotRefresh::Refreshed(_)));
        assert_eq!(slot(&conn, 301).scanning_code.as_deref(), Some("NEW-10"));
        for memory_no in [300, 302, 303] {
            assert_eq!(slot(&conn, memory_no).status, PluSlotStatus::Free);
        }
        assert_eq!(
            system_repo::get_setting(&conn, SNAPSHOT_SETTLEMENT_NO_KEY)
                .unwrap()
                .as_deref(),
            Some("10")
        );

        // 同じ精算回数で bytes の違う 2 本は照合しない。bytes が同じなら照合する
        let before = reconcile_state(&conn);
        let ambiguous = vec![
            snapshot_bytes("0011", &at(304, "A")),
            snapshot_bytes("0011", &at(305, "B")),
        ];
        assert!(matches!(
            refresh_plu_register_snapshot_from_sd(&mut conn, &ambiguous).unwrap(),
            PluRegisterSnapshotRefresh::Unchanged
        ));
        assert_eq!(reconcile_state(&conn), before);
        let same = vec![ambiguous[0].clone(), ambiguous[0].clone()];
        assert!(matches!(
            refresh_plu_register_snapshot_from_sd(&mut conn, &same).unwrap(),
            PluRegisterSnapshotRefresh::Refreshed(_)
        ));
        assert_eq!(slot(&conn, 304).scanning_code.as_deref(), Some("A"));
    }

    #[test]
    fn test_refresh_plu_register_snapshot_from_sd_req907_key_comparison() {
        // REQ-907 / 33 §16.3 手順 2〜3: key が無い・より大きいときだけ照合する。ほかは何も変えない
        let (_dir, mut conn) = setup_test_db();
        let file = |settlement_no: &str| {
            vec![snapshot_bytes(
                settlement_no,
                &[(300, format!("CODE-{settlement_no}"))],
            )]
        };
        // key が無い → 照合する
        assert!(matches!(
            refresh_plu_register_snapshot_from_sd(&mut conn, &file("0010")).unwrap(),
            PluRegisterSnapshotRefresh::Refreshed(_)
        ));

        type Case = (
            &'static str,
            Vec<Vec<u8>>,
            fn(&PluRegisterSnapshotRefresh) -> bool,
        );
        let unchanged: fn(&PluRegisterSnapshotRefresh) -> bool =
            |refresh| matches!(refresh, PluRegisterSnapshotRefresh::Unchanged);
        let cases: [Case; 4] = [
            ("等しい", file("0010"), unchanged),
            ("小さい", file("0009"), |refresh| {
                matches!(refresh, PluRegisterSnapshotRefresh::SkippedOlder)
            }),
            ("使える Z004 が無い", vec![b"garbage".to_vec()], unchanged),
            ("Z004 が 0 本", Vec::new(), unchanged),
        ];
        for (label, files, expected) in cases {
            let before = reconcile_state(&conn);
            let refresh = refresh_plu_register_snapshot_from_sd(&mut conn, &files).unwrap();
            assert!(expected(&refresh), "{label}: {refresh:?}");
            assert_eq!(reconcile_state(&conn), before, "{label}");
        }

        // より大きい → 照合する
        assert!(matches!(
            refresh_plu_register_snapshot_from_sd(&mut conn, &file("0011")).unwrap(),
            PluRegisterSnapshotRefresh::Refreshed(_)
        ));
        assert_eq!(slot(&conn, 300).scanning_code.as_deref(), Some("CODE-0011"));
        assert_eq!(
            system_repo::get_setting(&conn, SNAPSHOT_SETTLEMENT_NO_KEY)
                .unwrap()
                .as_deref(),
            Some("11")
        );

        // key が整数として読めない → 大きい精算回数でも照合しない
        system_repo::upsert_setting(&conn, SNAPSHOT_SETTLEMENT_NO_KEY, "1x").unwrap();
        let before = reconcile_state(&conn);
        assert!(unchanged(
            &refresh_plu_register_snapshot_from_sd(&mut conn, &file("0099")).unwrap()
        ));
        assert_eq!(reconcile_state(&conn), before);
    }

    #[test]
    fn test_prepare_plu_export_req907_uses_min_free_and_sticky_allocation() {
        // REQ-907 / PLU-25: 最小の空きを取り、confirm の前の書き出し直し（Diff の後の Full）は同じ memory No.
        let (_dir, mut conn) = setup_test_db();
        let jan_a = valid_jan_from_seed(101);
        let jan_b = valid_jan_from_seed(102);
        seed_product_for_plu_with_jan(&conn, "ALLOC-A", Some(&jan_a), "A", 1, false, true);
        seed_product_for_plu_with_jan(&conn, "ALLOC-B", Some(&jan_b), "B", 1, false, true);
        conn.execute(
            "UPDATE plu_slots SET status='external', scanning_code='EXT-' || memory_no",
            [],
        )
        .unwrap();
        for memory_no in [217, 300, 401, 5_000] {
            put_slot(&conn, memory_no, None, PluSlotStatus::Free);
        }

        let first = prepare(&mut conn, ExportMode::Diff);
        assert_eq!(memory_numbers(&first), vec![217, 300]);
        let second = prepare(&mut conn, ExportMode::Full);
        assert_eq!(memory_numbers(&second), vec![217, 300]);
        assert_eq!(second.prepared_rows, first.prepared_rows);
        let reserved: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM plu_slots WHERE status='reserved'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(reserved, 2);
    }

    #[test]
    fn test_prepare_plu_export_req907_adopts_min_external_before_free() {
        // REQ-907 / 33 §16.4 手順 5: 同じコードの external があれば、最小の番号を採用して新しく予約しない
        let (_dir, mut conn) = setup_test_db();
        let jan = valid_jan_from_seed(111);
        seed_product_for_plu_with_jan(&conn, "ADOPT", Some(&jan), "adopt", 1, false, true);
        put_slot(&conn, 300, Some(&jan), PluSlotStatus::External);
        put_slot(&conn, 218, Some(&jan), PluSlotStatus::External);
        let free_before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM plu_slots WHERE status='free'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let result = prepare(&mut conn, ExportMode::Diff);

        assert_eq!(memory_numbers(&result), vec![218]);
        assert_eq!(slot(&conn, 218).status, PluSlotStatus::Active);
        assert!(slot(&conn, 218).activated_at.is_some());
        assert_eq!(slot(&conn, 300).status, PluSlotStatus::External);
        let free_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM plu_slots WHERE status='free'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(free_after, free_before);
    }

    #[test]
    fn test_prepare_plu_export_req907_reallocates_after_reservation_dropped() {
        // REQ-907 / B-F2: register 側で失われた予約 memory は external のまま再利用しない。
        let (_dir, mut conn) = setup_test_db();
        let jan = valid_jan_from_seed(116);
        seed_product_for_plu_with_jan(
            &conn,
            "RESERVATION-DROPPED",
            Some(&jan),
            "reservation dropped",
            1,
            false,
            true,
        );
        put_slot(&conn, 300, Some(&jan), PluSlotStatus::Reserved);
        import_plu_register_snapshot(
            &mut conn,
            &snapshot_bytes("0001", &[(300, "DIFFERENT-SYNTH".to_string())]),
            1,
        )
        .unwrap();
        assert_eq!(slot(&conn, 300).status, PluSlotStatus::External);

        let prepared = prepare(&mut conn, ExportMode::Diff);

        assert_eq!(
            memory_numbers(&prepared),
            vec![217],
            "再予約は最小の free slot を使う"
        );
        let replacement = slot(&conn, 217);
        assert_eq!(replacement.status, PluSlotStatus::Reserved);
        assert_eq!(replacement.scanning_code.as_deref(), Some(jan.as_str()));
        assert_eq!(slot(&conn, 300).status, PluSlotStatus::External);
    }

    #[test]
    fn test_confirm_plu_export_req907_activates_reserved_slot_and_is_idempotent() {
        // REQ-907 / 33 §16.5: reserved → active。同じ結果への再 confirm は成功し、別の枠は拒む
        let (_dir, mut conn) = setup_test_db();
        let jan = valid_jan_from_seed(121);
        seed_product_for_plu_with_jan(&conn, "CONFIRM-A", Some(&jan), "confirm", 1, false, true);
        put_slot(&conn, 217, Some(&jan), PluSlotStatus::Reserved);
        let row = |memory_no| PluPreparedRow {
            memory_no,
            target_product_codes: vec!["CONFIRM-A".to_string()],
            content_hash: current_hash(&conn, memory_no, "CONFIRM-A"),
        };
        let req = PluExportConfirmRequest {
            product_codes: vec!["CONFIRM-A".to_string()],
            prepared_rows: vec![row(217)],
        };
        let other_slot = PluExportConfirmRequest {
            product_codes: vec!["CONFIRM-A".to_string()],
            prepared_rows: vec![row(219)],
        };

        let result = confirm_plu_export_saved(&mut conn, req.clone()).unwrap();

        assert_eq!(result.updated_count, 1);
        assert!(result.remaining_products.is_empty());
        assert_eq!(slot(&conn, 217).status, PluSlotStatus::Active);
        assert!(!plu_dirty(&conn, "CONFIRM-A"));
        confirm_plu_export_saved(&mut conn, req).unwrap();
        assert!(matches!(
            confirm_plu_export_saved(&mut conn, other_slot),
            Err(BizError::ValidationFailed(_))
        ));
    }

    #[test]
    fn test_prepare_plu_export_req907_no_free_slot_excludes_only_affected_groups() {
        // REQ-907: A-P4 non-empty output paired with NoFreeSlot exclusions.
        let (_dir, mut conn) = setup_test_db();
        let active_jan = valid_jan_from_seed(131);
        seed_product_for_plu_with_jan(
            &conn,
            "HAS-SLOT",
            Some(&active_jan),
            "active",
            1,
            false,
            true,
        );
        seed_product_for_plu(&conn, "NO-SLOT-A", "A", 1, false, true);
        seed_product_for_plu(&conn, "NO-SLOT-B", "B", 1, false, true);
        conn.execute(
            "UPDATE plu_slots SET status='external', scanning_code='EXT-' || memory_no",
            [],
        )
        .unwrap();
        put_slot(&conn, 217, Some(&active_jan), PluSlotStatus::Active);

        let result = prepare(&mut conn, ExportMode::Full);
        assert_eq!(memory_numbers(&result), vec![217]);
        let no_free: Vec<_> = result
            .excluded
            .iter()
            .filter(|item| item.reason == PluExcludedReason::NoFreeSlot)
            .collect();
        assert_eq!(no_free.len(), 2);
    }

    #[test]
    fn test_prepare_plu_export_req907_never_emits_rows_for_untargeted_slots() {
        // REQ-907 / 33 §16.4 手順 6・D-117 (1): 対象の商品を持たない JAN の枠・external・free の行も、clear 行も出さない
        let (_dir, mut conn) = setup_test_db();
        let jan = |seed| valid_jan_from_seed(seed);
        let (dirty_jan, clean_jan, off_jan, discontinued_jan, orphan_jan) =
            (jan(141), jan(142), jan(145), jan(146), jan(147));
        seed_product_for_plu_with_jan(&conn, "ROW-DIRTY", Some(&dirty_jan), "d", 1, false, true);
        seed_product_for_plu_with_jan(&conn, "ROW-CLEAN", Some(&clean_jan), "c", 1, false, false);
        seed_product_for_plu_with_jan(&conn, "ROW-OFF", Some(&off_jan), "o", 1, false, true);
        set_plu_state(&conn, "ROW-OFF", false, true);
        seed_product_for_plu_with_jan(
            &conn,
            "ROW-DISC",
            Some(&discontinued_jan),
            "x",
            1,
            true,
            true,
        );
        put_slot(&conn, 300, Some(&dirty_jan), PluSlotStatus::Reserved);
        put_slot(&conn, 217, Some(&clean_jan), PluSlotStatus::Active);
        put_slot(&conn, 401, Some(&off_jan), PluSlotStatus::Active);
        put_slot(&conn, 402, Some("EXTERNAL"), PluSlotStatus::External);
        put_slot(&conn, 403, Some(&discontinued_jan), PluSlotStatus::Active);
        put_slot(&conn, 404, Some(&orphan_jan), PluSlotStatus::Reserved);
        let slots_before = crate::db::test_support::dump(&conn, "SELECT * FROM plu_slots");

        let full = prepare(&mut conn, ExportMode::Full);
        let diff = prepare(&mut conn, ExportMode::Diff);

        assert_eq!(memory_numbers(&full), vec![217, 300]);
        assert_eq!(memory_numbers(&diff), vec![300]);
        for (result, expected_lines) in [(&full, 3), (&diff, 2)] {
            let (text, _, _) = encoding_rs::SHIFT_JIS.decode(&result.plu_output.bytes);
            assert_eq!(text.lines().count(), expected_lines);
            assert!(!text.contains("00000000000000"), "clear 行が無い");
            for code in [&off_jan, &discontinued_jan, &orphan_jan] {
                assert!(!text.contains(code.as_str()));
            }
        }
        assert_eq!(
            crate::db::test_support::dump(&conn, "SELECT * FROM plu_slots"),
            slots_before
        );
    }

    #[test]
    fn test_prepare_plu_export_req907_keeps_invalid_jan_slot_without_output() {
        // REQ-907: A-E6 invalid JAN is excluded without output for its slot.
        let (_dir, mut conn) = setup_test_db();
        let invalid_jan = "4901234567890";
        let valid_jan = valid_jan_from_seed(143);
        seed_product_for_plu_with_jan(
            &conn,
            "INVALID-SLOT",
            Some(invalid_jan),
            "invalid",
            1,
            false,
            true,
        );
        seed_product_for_plu_with_jan(
            &conn,
            "VALID-SLOT",
            Some(&valid_jan),
            "valid",
            1,
            false,
            true,
        );
        put_slot(&conn, 217, Some(invalid_jan), PluSlotStatus::Active);
        put_slot(&conn, 218, Some(&valid_jan), PluSlotStatus::Active);

        for mode in [ExportMode::Full, ExportMode::Diff] {
            let result = prepare(&mut conn, mode);
            assert_eq!(memory_numbers(&result), vec![218]);
            assert!(result.excluded.iter().any(|item| {
                item.product_code == "INVALID-SLOT"
                    && item.reason == PluExcludedReason::InvalidCheckDigit
            }));
            let slot = slot(&conn, 217);
            assert_eq!(slot.status, PluSlotStatus::Active);
            assert_eq!(slot.scanning_code.as_deref(), Some(invalid_jan));
        }
    }

    fn import_row(product_code: &str, jan: &str) -> crate::biz::product_service::ImportRow {
        crate::biz::product_service::ImportRow {
            line_no: 2,
            product_code: product_code.to_string(),
            name: "new".to_string(),
            department_id: 1,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            stock_unit: Some("pcs".to_string()),
            initial_stock: None,
            jan_code: Some(jan.to_string()),
            maker_code: None,
            supplier_id: None,
            pos_stock_sync: Some(true),
            plu_target: None,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn test_product_service_req907_keeps_slot_on_untarget_discontinue_and_jan_change() {
        // REQ-907 / BIZ-04-D6（33 §16.6、D-117 (1)）: 対象外化・廃番化・JAN の変更で枠の状態を変えない
        use crate::biz::product_service::{self, ProductUpdateRequest};
        let (_dir, mut conn) = setup_test_db();
        let jan = |seed| valid_jan_from_seed(seed);
        let (off_active, off_reserved, discontinued, old_jan, new_jan) =
            (jan(151), jan(152), jan(161), jan(171), jan(172));
        for (code, jan_code) in [
            ("OFF-ACTIVE", &off_active),
            ("OFF-RESERVED", &off_reserved),
            ("DISC-SLOT", &discontinued),
            ("JAN-CHANGE", &old_jan),
        ] {
            seed_product_for_plu_with_jan(&conn, code, Some(jan_code), code, 1, false, false);
        }
        put_slot(&conn, 217, Some(&off_active), PluSlotStatus::Active);
        put_slot(&conn, 218, Some(&off_reserved), PluSlotStatus::Reserved);
        put_slot(&conn, 219, Some(&discontinued), PluSlotStatus::Active);
        put_slot(&conn, 220, Some(&old_jan), PluSlotStatus::Active);
        let slots_before = crate::db::test_support::dump(&conn, "SELECT * FROM plu_slots");

        for code in ["OFF-ACTIVE", "OFF-RESERVED"] {
            product_service::update_product(
                &mut conn,
                code,
                &ProductUpdateRequest {
                    plu_target: Some(false),
                    ..Default::default()
                },
            )
            .unwrap();
        }
        assert!(product_service::toggle_discontinue(&mut conn, "DISC-SLOT").unwrap());
        assert!(!product_service::toggle_discontinue(&mut conn, "DISC-SLOT").unwrap());
        product_service::commit_import(
            &mut conn,
            vec![import_row("JAN-CHANGE", &new_jan)],
            vec!["JAN-CHANGE".to_string()],
        )
        .unwrap();

        assert_eq!(
            crate::db::test_support::dump(&conn, "SELECT * FROM plu_slots"),
            slots_before
        );
        // 廃番解除は plu_target を戻さない。JAN の変更（商品 CSV の上書き）は未反映にする
        let product = |code| {
            product_repo::find_by_product_code(&conn, code)
                .unwrap()
                .unwrap()
                .product
        };
        assert!(!product("DISC-SLOT").plu_target);
        assert!(product("JAN-CHANGE").plu_target && product("JAN-CHANGE").plu_dirty);

        // 新しい JAN は次の prepare で別の枠を取り、前の JAN の枠は書き出さない
        let prepared = prepare(&mut conn, ExportMode::Diff);
        assert_eq!(memory_numbers(&prepared), vec![221]);
        assert_eq!(
            slot(&conn, 221).scanning_code.as_deref(),
            Some(new_jan.as_str())
        );
        assert_eq!(slot(&conn, 220).status, PluSlotStatus::Active);
    }

    #[test]
    fn test_prepare_plu_export_req907_retarget_reuses_same_memory_no() {
        // REQ-907 / 33 §16.4 手順 4: 対象外にした後に対象へ戻した商品は、前と同じ枠を使う
        use crate::biz::product_service::{self, ProductUpdateRequest};
        let (_dir, mut conn) = setup_test_db();
        let jan = valid_jan_from_seed(181);
        seed_product_for_plu_with_jan(&conn, "RETARGET", Some(&jan), "r", 1, false, false);
        put_slot(&conn, 300, Some(&jan), PluSlotStatus::Active);
        for plu_target in [false, true] {
            product_service::update_product(
                &mut conn,
                "RETARGET",
                &ProductUpdateRequest {
                    plu_target: Some(plu_target),
                    ..Default::default()
                },
            )
            .unwrap();
        }

        for mode in [ExportMode::Diff, ExportMode::Full] {
            assert_eq!(memory_numbers(&prepare(&mut conn, mode)), vec![300]);
        }
        assert_eq!(slot(&conn, 217).status, PluSlotStatus::Free);
    }

    #[test]
    fn test_prepare_plu_export_req907_shared_jan_partial_change_excluded_in_diff_and_full() {
        // REQ-907 / 33 §16.4 手順 2・3: 群はその JAN の対象の商品の全員で作る。
        // 共有 JAN の片方だけ税率を変えると、Diff も Full も group_price_mismatch で除外する
        use crate::biz::product_service::{self, ProductUpdateRequest};
        use crate::db::product_repo::ProductTaxRate;
        let (_dir, mut conn) = setup_test_db();
        let shared = valid_jan_from_seed(191);
        let other = valid_jan_from_seed(192);
        seed_product_for_plu_with_jan(&conn, "SHARED-A", Some(&shared), "A", 1, false, false);
        seed_product_for_plu_with_jan(&conn, "SHARED-B", Some(&shared), "B", 1, false, false);
        seed_product_for_plu_with_jan(&conn, "OTHER", Some(&other), "O", 1, false, true);
        put_slot(&conn, 300, Some(&shared), PluSlotStatus::Active);
        product_service::update_product(
            &mut conn,
            "SHARED-B",
            &ProductUpdateRequest {
                tax_rate: Some(ProductTaxRate::Rate8),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(plu_dirty(&conn, "SHARED-B") && !plu_dirty(&conn, "SHARED-A"));

        for mode in [ExportMode::Diff, ExportMode::Full] {
            let result = prepare(&mut conn, mode);
            assert_eq!(memory_numbers(&result), vec![217], "{mode:?}");
            assert_eq!(result.target_product_codes, vec!["OTHER".to_string()]);
            let mismatched: Vec<&str> = result
                .excluded
                .iter()
                .filter(|item| item.reason == PluExcludedReason::GroupPriceMismatch)
                .map(|item| item.product_code.as_str())
                .collect();
            assert_eq!(mismatched, vec!["SHARED-A", "SHARED-B"], "{mode:?}");
        }
        assert_eq!(slot(&conn, 300).status, PluSlotStatus::Active);
    }

    #[test]
    fn test_confirm_plu_export_saved_req907_clears_only_rows_with_unchanged_content() {
        // REQ-907 / 33 §16.5（D-160 (5)）: 保存の後に出力の行が変わった商品は未反映のまま残す。
        // 合否は IO-04 が書く行の byte 列だけで決める（CP932 の 16 バイトで落ちる末尾だけの変更は同じ行）
        use crate::biz::product_service::{self, ProductUpdateRequest};
        use sha2::{Digest, Sha256};
        let (_dir, mut conn) = setup_test_db();
        let jan = |seed| valid_jan_from_seed(seed);
        let (head_jan, tail_jan, same_jan) = (jan(201), jan(202), jan(203));
        seed_product_for_plu_with_jan(&conn, "ROW-HEAD", Some(&head_jan), "A-HEAD", 1, false, true);
        seed_product_for_plu_with_jan(
            &conn,
            "ROW-TAIL",
            Some(&tail_jan),
            "ABCDEFGHIJKLMNOPQRS",
            1,
            false,
            true,
        );
        seed_product_for_plu_with_jan(&conn, "ROW-SAME", Some(&same_jan), "SAME", 1, false, true);

        let prepared = prepare(&mut conn, ExportMode::Diff);
        assert_eq!(memory_numbers(&prepared), vec![217, 218, 219]);
        // 照合の値の独立の期待値: file の 1 行（11 列、CRLF なし）の CP932 の byte 列の SHA-256
        let line = format!(
            "000219\t{same_jan}\tSAME            \t500\t税1(内税)\tいいえ\tいいえ\tいいえ\tいいえ\t無し\tその他小物"
        );
        let (line_bytes, _, _) = encoding_rs::SHIFT_JIS.encode(&line);
        assert_eq!(
            prepared.prepared_rows[2].content_hash,
            format!("{:x}", Sha256::digest(&line_bytes))
        );

        // 保存の後: 名称の先頭の 1 文字を変える／切り詰めで落ちる末尾（17 バイト目より後）だけを変える
        for (code, name) in [("ROW-HEAD", "B-HEAD"), ("ROW-TAIL", "ABCDEFGHIJKLMNOPQRX")] {
            product_service::update_product(
                &mut conn,
                code,
                &ProductUpdateRequest {
                    name: Some(name.to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        }

        let result = confirm_plu_export_saved(
            &mut conn,
            PluExportConfirmRequest {
                product_codes: prepared.target_product_codes,
                prepared_rows: prepared.prepared_rows,
            },
        )
        .unwrap();

        assert_eq!(
            result.remaining_products,
            vec![PluRemainingProduct {
                product_code: "ROW-HEAD".to_string(),
                name: "B-HEAD".to_string(),
            }]
        );
        assert_eq!(result.updated_count, 2);
        assert!(plu_dirty(&conn, "ROW-HEAD"));
        assert!(!plu_dirty(&conn, "ROW-TAIL") && !plu_dirty(&conn, "ROW-SAME"));
        assert_eq!(slot(&conn, 217).status, PluSlotStatus::Reserved);
        assert_eq!(slot(&conn, 218).status, PluSlotStatus::Active);
        assert_eq!(slot(&conn, 219).status, PluSlotStatus::Active);
    }

    #[test]
    fn test_prepare_plu_export_req402_does_not_update_dirty_or_exported_at() {
        // REQ-402 / D-027: PLUファイル生成だけでは未反映を解除しない
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu(&conn, "PLU-P01", "準備対象A", 1, false, true);
        seed_product_for_plu(&conn, "PLU-P02", "準備対象B", 1, false, true);

        let req = PluExportPrepareRequest {
            mode: ExportMode::Diff,
        };
        let result = prepare_plu_export(&mut conn, req).unwrap();

        assert_eq!(result.count, 2);
        assert_eq!(
            result.target_product_codes,
            vec!["PLU-P01".to_string(), "PLU-P02".to_string()]
        );
        assert!(!result.plu_output.bytes.is_empty());

        for code in ["PLU-P01", "PLU-P02"] {
            let product = product_repo::find_by_product_code(&conn, code)
                .unwrap()
                .unwrap();
            assert!(
                product.product.plu_dirty,
                "{code} remains dirty after prepare"
            );
            assert!(
                product.product.plu_exported_at.is_none(),
                "{code} exported_at remains unset after prepare"
            );
        }
    }

    #[test]
    fn test_confirm_plu_export_saved_req402_updates_only_requested_products() {
        // REQ-402 / D-027: 保存後確認した exact product_code set だけ未反映解除する
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu(&conn, "PLU-CF1", "確認対象A", 1, false, true);
        seed_product_for_plu(&conn, "PLU-CF2", "確認対象B", 1, false, true);
        seed_product_for_plu(&conn, "PLU-CF3", "未確認対象", 1, false, true);

        let req = PluExportConfirmRequest {
            product_codes: vec!["PLU-CF1".to_string(), "PLU-CF2".to_string()],
            prepared_rows: prepared_rows_for_codes(
                &conn,
                &["PLU-CF1".to_string(), "PLU-CF2".to_string()],
            ),
        };
        let result = confirm_plu_export_saved(&mut conn, req).unwrap();

        assert_eq!(result.updated_count, 2);
        assert!(result.confirmed_at.contains('T'));

        for code in ["PLU-CF1", "PLU-CF2"] {
            let product = product_repo::find_by_product_code(&conn, code)
                .unwrap()
                .unwrap();
            assert!(!product.product.plu_dirty, "{code} was confirmed");
            assert!(
                product.product.plu_exported_at.is_some(),
                "{code} exported_at was set"
            );
        }

        let untouched = product_repo::find_by_product_code(&conn, "PLU-CF3")
            .unwrap()
            .unwrap();
        assert!(
            untouched.product.plu_dirty,
            "unconfirmed product remains dirty"
        );
        assert!(
            untouched.product.plu_exported_at.is_none(),
            "unconfirmed product exported_at remains unset"
        );
    }

    #[test]
    fn test_confirm_plu_export_saved_req402_accepts_target_codes_exceeding_row_limit() {
        // REQ-402 / REQ-907: confirm は商品件数ではなく prepared row の exact set を検証する。
        let (_dir, mut conn) = setup_test_db();
        let product_codes: Vec<String> = (0..=4_784)
            .map(|idx| format!("PLU-LIMIT-{idx:04}"))
            .collect();
        let shared_jan = valid_jan_from_seed(9_999);

        {
            let tx = conn.transaction().unwrap();
            for code in &product_codes {
                seed_product_for_plu_with_jan(&tx, code, Some(&shared_jan), code, 1, false, true);
            }
            tx.commit().unwrap();
        }

        plu_slot_repo::update_slot(
            &conn,
            PluSlotUpdate {
                memory_no: 217,
                scanning_code: Some(&shared_jan),
                status: PluSlotStatus::Reserved,
                reserved_at: Some("2026-08-18T00:00:00"),
                activated_at: None,
                updated_at: "2026-08-18T00:00:00",
            },
        )
        .unwrap();
        let prepared_rows = vec![PluPreparedRow {
            memory_no: 217,
            target_product_codes: product_codes.clone(),
            content_hash: current_hash(&conn, 217, &product_codes[0]),
        }];
        let result = confirm_plu_export_saved(
            &mut conn,
            PluExportConfirmRequest {
                product_codes: product_codes.clone(),
                prepared_rows,
            },
        )
        .unwrap();

        assert_eq!(result.updated_count, product_codes.len());
        for code in [
            product_codes.first().unwrap(),
            product_codes.last().unwrap(),
        ] {
            let product = product_repo::find_by_product_code(&conn, code)
                .unwrap()
                .unwrap();
            assert!(!product.product.plu_dirty, "{code} was confirmed");
            assert!(
                product.product.plu_exported_at.is_some(),
                "{code} exported_at was set"
            );
        }
    }

    #[test]
    fn test_confirm_plu_export_saved_req402_rejects_invalid_sets_and_rolls_back() {
        // REQ-402 / D-027: 空・重複・欠番は拒否し、途中更新しない
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu(&conn, "PLU-RB1", "ロールバックA", 1, false, true);
        seed_product_for_plu(&conn, "PLU-RB2", "ロールバックB", 1, false, true);

        let empty = confirm_plu_export_saved(
            &mut conn,
            PluExportConfirmRequest {
                product_codes: vec![],
                prepared_rows: vec![],
            },
        );
        assert!(matches!(empty, Err(BizError::ValidationFailed(_))));

        let duplicate_rows = prepared_rows_for_codes(&conn, &["PLU-RB1".to_string()]);
        let duplicate = confirm_plu_export_saved(
            &mut conn,
            PluExportConfirmRequest {
                product_codes: vec!["PLU-RB1".to_string(), "PLU-RB1".to_string()],
                prepared_rows: duplicate_rows,
            },
        );
        assert!(matches!(duplicate, Err(BizError::ValidationFailed(_))));

        let missing_rows =
            prepared_rows_for_codes(&conn, &["PLU-RB1".to_string(), "NO-SUCH-PLU".to_string()]);
        let missing = confirm_plu_export_saved(
            &mut conn,
            PluExportConfirmRequest {
                product_codes: vec!["PLU-RB1".to_string(), "NO-SUCH-PLU".to_string()],
                prepared_rows: missing_rows,
            },
        );
        assert!(matches!(missing, Err(BizError::NotFound(_))));

        for code in ["PLU-RB1", "PLU-RB2"] {
            let product = product_repo::find_by_product_code(&conn, code)
                .unwrap()
                .unwrap();
            assert!(
                product.product.plu_dirty,
                "{code} remains dirty after failed confirm"
            );
            assert!(
                product.product.plu_exported_at.is_none(),
                "{code} exported_at remains unset after failed confirm"
            );
        }
    }

    #[test]
    fn test_prepare_plu_export_req402_full_mode() {
        // REQ-402: PLU書出し
        // BIZ-04: Fullモードで全active商品が書出され、DB状態は更新されないこと
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu(&conn, "PLU-001", "テスト毛糸A", 1, false, true);
        seed_product_for_plu(&conn, "PLU-002", "テスト毛糸B", 1, false, false);
        seed_product_for_plu(&conn, "PLU-DISC", "廃番品", 1, true, true);

        let req = PluExportPrepareRequest {
            mode: ExportMode::Full,
        };
        let result = prepare_plu_export(&mut conn, req).unwrap();

        assert_eq!(result.count, 2, "廃番を除いた2件が書出される");
        assert_eq!(
            result.target_product_codes,
            vec!["PLU-001".to_string(), "PLU-002".to_string()]
        );
        assert!(!result.plu_output.bytes.is_empty());

        let p1 = product_repo::find_by_product_code(&conn, "PLU-001")
            .unwrap()
            .unwrap();
        assert!(p1.product.plu_dirty, "prepareではplu_dirtyを更新しない");
        assert!(p1.product.plu_exported_at.is_none());

        let p2 = product_repo::find_by_product_code(&conn, "PLU-002")
            .unwrap()
            .unwrap();
        assert!(
            !p2.product.plu_dirty,
            "元からplu_dirty=0だった商品はそのまま"
        );
        assert!(p2.product.plu_exported_at.is_none());
    }

    #[test]
    fn test_prepare_plu_export_req402_rate8_uses_tax2_internal() {
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu(&conn, "PLU-TAX8", "税率8商品", 1, false, true);
        conn.execute(
            "UPDATE products SET tax_rate = '8' WHERE product_code = 'PLU-TAX8'",
            [],
        )
        .unwrap();

        let result = prepare_plu_export(
            &mut conn,
            PluExportPrepareRequest {
                mode: ExportMode::Diff,
            },
        )
        .unwrap();
        let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(&result.plu_output.bytes);
        assert!(!had_errors);
        assert!(decoded.contains("税2(内税)"), "PLU output: {decoded}");
    }

    #[test]
    fn test_prepare_plu_export_req402_diff_mode() {
        // REQ-402: PLU書出し
        // BIZ-04: Diffモードでplu_dirty=1の商品のみ書出されること
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu(&conn, "PLU-D01", "ダーティ商品", 1, false, true);
        seed_product_for_plu(&conn, "PLU-C01", "クリーン商品", 1, false, false);

        let req = PluExportPrepareRequest {
            mode: ExportMode::Diff,
        };
        let result = prepare_plu_export(&mut conn, req).unwrap();

        assert_eq!(result.count, 1, "plu_dirty=1の1件のみ");
        assert_eq!(result.target_product_codes, vec!["PLU-D01".to_string()]);

        let clean = product_repo::find_by_product_code(&conn, "PLU-C01")
            .unwrap()
            .unwrap();
        assert!(!clean.product.plu_dirty);
        assert!(clean.product.plu_exported_at.is_none(), "未書出しのまま");
    }

    #[test]
    fn test_prepare_plu_export_req402_empty_returns_validation_error() {
        // REQ-402: PLU書出し
        // BIZ-04: 0件 → ValidationFailed
        let (_dir, mut conn) = setup_test_db();

        let req = PluExportPrepareRequest {
            mode: ExportMode::Diff,
        };
        let result = prepare_plu_export(&mut conn, req);

        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_plu_format_failure_req402_maps_to_export_error() {
        // REQ-402 / BIZ-04-D2: formatter failure は書出し専用 error 語彙へ変換する。
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu(&conn, "PLU-ERR", "合成商品😀", 1, false, true);

        let result = prepare_plu_export(
            &mut conn,
            PluExportPrepareRequest {
                mode: ExportMode::Diff,
            },
        );

        assert!(
            matches!(
                result,
                Err(BizError::ExportError(ref message))
                    if message.starts_with("PLUファイルの生成に失敗しました:")
            ),
            "unexpected result: {result:?}"
        );
    }

    #[test]
    fn test_prepare_plu_export_req402_excludes_products_without_valid_13_digit_jan() {
        // REQ-402 / D-028 / CV17 1.1.1: JAN不備商品は全体失敗ではなく対象外リストへ分離する。
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu(&conn, "JAN-OK", "JAN正常", 1, false, true);
        seed_product_for_plu_with_jan(&conn, "JAN-NONE", None, "JANなし", 1, false, true);
        seed_product_for_plu_with_jan(
            &conn,
            "JAN-SHORT",
            Some("12345678"),
            "JAN短い",
            1,
            false,
            true,
        );
        seed_product_for_plu_with_jan(
            &conn,
            "JAN-BAD",
            Some("4901234567890"),
            "JAN検査桁不正",
            1,
            false,
            true,
        );

        let result = prepare_plu_export(
            &mut conn,
            PluExportPrepareRequest {
                mode: ExportMode::Diff,
            },
        )
        .unwrap();

        assert_eq!(result.count, 1);
        assert_eq!(result.target_product_codes, vec!["JAN-OK".to_string()]);
        assert_eq!(result.excluded.len(), 3);
        assert!(result.excluded.iter().any(|excluded| {
            excluded.product_code == "JAN-NONE"
                && matches!(excluded.reason, PluExcludedReason::MissingJan)
        }));
        assert!(result.excluded.iter().any(|excluded| {
            excluded.product_code == "JAN-SHORT"
                && matches!(excluded.reason, PluExcludedReason::InvalidJanFormat)
        }));
        assert!(result.excluded.iter().any(|excluded| {
            excluded.product_code == "JAN-BAD"
                && matches!(excluded.reason, PluExcludedReason::InvalidCheckDigit)
        }));
    }

    #[test]
    fn test_prepare_plu_export_req402_fails_when_all_targets_are_excluded_with_details() {
        // REQ-402 / D-028: 全件が要修正の場合は、商品コードと理由をValidationFailedに含める。
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu_with_jan(&conn, "JAN-NONE", None, "JANなし", 1, false, true);
        seed_product_for_plu_with_jan(
            &conn,
            "JAN-SHORT",
            Some("12345678"),
            "JAN短い",
            1,
            false,
            true,
        );

        let result = prepare_plu_export(
            &mut conn,
            PluExportPrepareRequest {
                mode: ExportMode::Diff,
            },
        );

        match result {
            Err(BizError::ValidationFailed(message)) => {
                assert!(message.contains("PLUファイルに書き出せる商品がありません"));
                assert!(message.contains("商品マスタで13桁JANを確認してください"));
                assert!(message.contains("JAN-NONE（JAN未登録）"));
                assert!(message.contains("JAN-SHORT（JANが13桁ではありません）"));
            }
            other => panic!("ValidationFailed が期待されるが {:?} が返った", other),
        }
    }

    #[test]
    fn test_prepare_plu_export_req402_deduplicates_same_jan_and_confirm_clears_group() {
        // REQ-402 / D-028: 同一JAN同価格は代表1行、confirm対象はグループ全員を解除する。
        let (_dir, mut conn) = setup_test_db();
        let same_jan = valid_jan_from_seed(7001);
        seed_product_for_plu_with_jan(
            &conn,
            "DEDUP-B",
            Some(&same_jan),
            "代表候補B",
            1,
            false,
            true,
        );
        seed_product_for_plu_with_jan(
            &conn,
            "DEDUP-A",
            Some(&same_jan),
            "代表候補A",
            1,
            false,
            true,
        );

        let mismatch_jan = valid_jan_from_seed(7002);
        seed_product_for_plu_with_jan(
            &conn,
            "MISMATCH-A",
            Some(&mismatch_jan),
            "不一致A",
            1,
            false,
            true,
        );
        seed_product_for_plu_with_jan(
            &conn,
            "MISMATCH-B",
            Some(&mismatch_jan),
            "不一致B",
            1,
            false,
            true,
        );
        product_repo::update_product(
            &conn,
            "MISMATCH-B",
            &ProductUpdates {
                selling_price: Some(700),
                ..Default::default()
            },
        )
        .unwrap();

        let result = prepare_plu_export(
            &mut conn,
            PluExportPrepareRequest {
                mode: ExportMode::Diff,
            },
        )
        .unwrap();

        assert_eq!(result.count, 1);
        assert_eq!(
            result.target_product_codes,
            vec!["DEDUP-A".to_string(), "DEDUP-B".to_string()]
        );
        assert!(!result.plu_output.bytes.is_empty());
        assert!(result.excluded.iter().any(|excluded| {
            excluded.product_code == "MISMATCH-A"
                && matches!(excluded.reason, PluExcludedReason::GroupPriceMismatch)
        }));
        assert!(result.excluded.iter().any(|excluded| {
            excluded.product_code == "MISMATCH-B"
                && matches!(excluded.reason, PluExcludedReason::GroupPriceMismatch)
        }));

        confirm_plu_export_saved(
            &mut conn,
            PluExportConfirmRequest {
                product_codes: result.target_product_codes,
                prepared_rows: result.prepared_rows,
            },
        )
        .unwrap();

        let dedup_a = product_repo::find_by_product_code(&conn, "DEDUP-A")
            .unwrap()
            .unwrap();
        let dedup_b = product_repo::find_by_product_code(&conn, "DEDUP-B")
            .unwrap()
            .unwrap();
        assert!(!dedup_a.product.plu_dirty, "DEDUP-A should be cleared");
        assert!(!dedup_b.product.plu_dirty, "DEDUP-B should be cleared");
    }

    #[test]
    fn test_confirm_plu_export_saved_req402_exported_at_updated() {
        // REQ-402: PLU書出し
        // BIZ-04: 保存済み確認で plu_exported_at が更新されること
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu(&conn, "PLU-AT1", "タイムスタンプ確認", 1, false, true);

        let prepared_rows = prepared_rows_for_codes(&conn, &["PLU-AT1".to_string()]);
        confirm_plu_export_saved(
            &mut conn,
            PluExportConfirmRequest {
                product_codes: vec!["PLU-AT1".to_string()],
                prepared_rows,
            },
        )
        .unwrap();

        let p = product_repo::find_by_product_code(&conn, "PLU-AT1")
            .unwrap()
            .unwrap();
        assert!(p.product.plu_exported_at.is_some());
        // ISO 8601形式であること
        let exported_at = p.product.plu_exported_at.unwrap();
        assert!(exported_at.contains('T'), "ISO 8601形式: {}", exported_at);
    }

    #[test]
    fn test_confirm_plu_export_saved_req402_operation_log_recorded() {
        // REQ-402: PLU書出し
        // BIZ-04: 保存済み確認の operation_log が記録されること
        let (_dir, mut conn) = setup_test_db();
        seed_product_for_plu(&conn, "PLU-LOG", "ログ確認", 1, false, true);

        let prepared_rows = prepared_rows_for_codes(&conn, &["PLU-LOG".to_string()]);
        confirm_plu_export_saved(
            &mut conn,
            PluExportConfirmRequest {
                product_codes: vec!["PLU-LOG".to_string()],
                prepared_rows,
            },
        )
        .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'plu_export'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "plu_export ログが1件記録される");

        let detail_json: String = conn
            .query_row(
                "SELECT detail_json FROM operation_logs WHERE operation_type = 'plu_export'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(detail_json.contains(r#""count":1"#));
        assert!(detail_json.contains(r#""confirmed_at":"#));
    }

    #[test]
    fn test_prepare_plu_export_req402_scanning_plu_limit() {
        // REQ-402 / REQ-907: 固定件数で全体を拒否せず、空き不足分だけ no_free_slot にする。
        let (_dir, mut conn) = setup_test_db();
        for idx in 0..=4_784 {
            let code = format!("OL-{:04}", idx);
            let jan = valid_jan_from_seed(idx as u64);
            seed_product_for_plu_with_jan(&conn, &code, Some(&jan), "上限確認", 1, false, true);
        }

        let req = PluExportPrepareRequest {
            mode: ExportMode::Full,
        };
        let result = prepare_plu_export(&mut conn, req).unwrap();
        assert_eq!(result.prepared_rows.len(), 4_784);
        assert_eq!(result.excluded.len(), 1);
        assert!(matches!(
            result.excluded[0].reason,
            PluExcludedReason::NoFreeSlot
        ));
    }

    #[test]
    fn test_list_plu_dirty_req402() {
        // REQ-402: PLU書出し
        // BIZ-04: list_plu_dirty は plu_dirty=1 の商品リストを返す
        let (_dir, conn) = setup_test_db();
        seed_product_for_plu(&conn, "LD-001", "ダーティA", 1, false, true);
        seed_product_for_plu(&conn, "LD-002", "ダーティB", 1, false, true);
        seed_product_for_plu(&conn, "LD-003", "クリーン", 1, false, false);

        let result = list_plu_dirty(&conn).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].product_code, "LD-001");
        assert_eq!(result[1].product_code, "LD-002");
    }
}
