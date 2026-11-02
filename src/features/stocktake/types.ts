export interface StocktakeSearch {
  dept?: number;
  counted_only?: boolean;
  page?: number;
  /** 73「今の数を入れる」: 商品を受けて開く（商品コード） */
  product?: string;
  /** 73「今の数を入れる」: 保存の後に戻る先（`/products/{product}/edit` だけを受ける） */
  returnTo?: string;
}
