// IO層: ファイルI/O・フォーマット変換（DB非依存の純関数群）

pub mod daily_report_parser;
pub mod ej_parser;
pub mod image_manager;
#[cfg(test)]
mod l3_fixtures;
pub mod plu_formatter;
pub mod pos_source_copy;
pub mod product_csv_importer;
pub mod register_sd;
pub mod report_csv_exporter;
pub mod z004_parser;

/// レジの SD の機種の folder（drive 直下からの 2 つの要素）。IO-09 と 71 MNT-01-D10 が共有する
/// 唯一の定義（29 §29.7.4 手順 1、D-114）。要素ごとに `join` し、区切りを含む 1 つの文字列にしない
pub(crate) const REGISTER_SD_FOLDER: [&str; 2] = ["CASIO", "SR500_550_4000"];
