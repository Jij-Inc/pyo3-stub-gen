# TODO: Python Stub構文からのメタデータ生成

## ✅ 実装完了 (Phase 1-3)

**Option A** と **Option C** の実装が完了しました！

### 実装済み機能

1. **Option A: `gen_function_from_python!` マクロ**
   - `submit!` ブロック内で使用
   - Python stub文字列から`PyFunctionInfo`を生成
   - オーバーロード定義などに使用

2. **Option C: `python` パラメータ**
   - `#[gen_stub_pyfunction(python = "...")]` 属性
   - 関数定義の近くに型情報を記述
   - より簡潔で読みやすい

### 使用例

```rust
// Option A
#[pyfunction]
fn func() { ... }

submit! {
    gen_function_from_python! {
        r#"def func(arg: SomeType) -> ReturnType: ..."#
    }
}

// Option C (推奨)
#[gen_stub_pyfunction(python = r#"
    def func(arg: SomeType) -> ReturnType: ...
"#)]
#[pyfunction]
fn func(arg: PyAny) { ... }
```

### テスト状況

- ✅ ユニットテスト: `insta`スナップショットテスト (6テストケース)
- ✅ 統合テスト: `examples/pure` (21/21 pytest パス)
- ✅ 型チェック: pyright エラーなし
- ✅ Lint: ruff エラーなし

---

## 背景

現在、型のオーバーロード（`@overload`）を実現するために、`submit!`マクロで手動で`PyFunctionInfo`や`PyMethodsInfo`を構築する必要がある。この方法は冗長で、Python開発者にとって直感的ではない。

### 現在の問題点

```rust
// examples/pure/src/manual_submit.rs の Incrementer
submit! {
    PyMethodsInfo {
        struct_id: std::any::TypeId::of::<Incrementer>,
        attrs: &[],
        getters: &[],
        setters: &[],
        methods: &[
            MethodInfo {
                name: "increment_1",
                args: &[
                    ArgInfo {
                        name: "x",
                        signature: None,
                        r#type: || i64::type_input(),
                    },
                ],
                r#type: MethodType::Instance,
                r#return: || i64::type_output(),
                doc: "And this is for the second comment",
                is_async: false,
                deprecated: None,
                type_ignored: None,
            }
        ],
    }
}
```

**問題:**
- 冗長で読みにくい
- TypeIdの手動指定が必要
- Pythonの型構文を知っていても、Rustの構造体に変換する必要がある
- エラーが発生しやすい

## 提案: Pythonスタブ構文の直接サポート

### 目標

Python開発者が慣れ親しんだ構文で型情報を書けるようにする。

### アプローチ: 2つの方法を提供

#### Option A: `submit!` + `gen_function_from_python!` (関数とRust実装が離れている場合)

```rust
// 関数の場合
submit! {
    gen_function_from_python! {
        r#"
            import collections.abc
            import typing

            def overload_example_1(x: int) -> int: ...
        "#
    }
}

// メソッドの場合
submit! {
    gen_methods_from_python! {
        class: Incrementer,
        method_stub: r#"
            def increment_1(self, x: int) -> int:
                """And this is for the second comment"""
        "#
    }
}
```

**適用ケース:**
- オーバーロードの追加型定義（`@overload`用）
- 関数とは別の場所でまとめて型定義を書きたい場合

#### Option C: 既存マクロの拡張 (attribute macro、推奨)

```rust
#[gen_stub_pyfunction(python = r#"
    import collections.abc
    import typing

    def fn_override_type(cb: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]: ...
"#)]
#[pyfunction]
fn fn_override_type<'a>(
    cb: Bound<'a, PyAny>,
) -> PyResult<Bound<'a, PyAny>> {
    cb.call1(("Hello!",))?;
    Ok(cb)
}
```

**適用ケース:**
- 関数定義の近くに型情報を書きたい場合
- 既存の `#[gen_stub(override_type)]` の代替として
- より簡潔で読みやすい

**メソッドの場合（要検討）:**
```rust
#[gen_stub_pymethods]
#[pymethods]
impl Incrementer {
    #[gen_stub_python(r#"
        def increment_1(self, x: int) -> int:
            """Additional overload"""
    "#)]
    fn increment_1(&self, x: f64) -> f64 { x + 1.0 }
}
```
※ 実装の複雑さ次第で対応

### 利点

1. **可読性**: Pythonの構文そのままなので直感的
2. **保守性**: `.pyi`ファイルとの一貫性が保たれる
3. **柔軟性**: 用途に応じて2つのアプローチから選択可能
4. **簡潔性**: Option Cは特に簡潔で関数定義の近くに書ける
5. **既存インフラ活用**: `submit!` と `inventory` をそのまま使える

## 設計上の重要な決定事項

### 決定: `submit!` をそのまま活用する方針

`submit!` マクロの仕組みは変更せず、**`PyMethodsInfo` を生成するproc-macro** を作る：

```rust
submit! {
    gen_methods_from_python! {
        class: Incrementer,
        method_stub: r#"
            def increment_1(self, x: int) -> int:
                """And this is for the second comment"""
        "#
    }
}
```

↓ **proc-macroで展開** ↓

```rust
submit! {
    PyMethodsInfo {
        struct_id: std::any::TypeId::of::<Incrementer>,
        attrs: &[],
        getters: &[],
        setters: &[],
        methods: &[
            MethodInfo {
                name: "increment_1",
                args: &[ArgInfo {
                    name: "x",
                    signature: None,
                    r#type: || i64::type_input(),
                }],
                r#type: MethodType::Instance,
                r#return: || i64::type_output(),
                doc: "And this is for the second comment",
                is_async: false,
                deprecated: None,
                type_ignored: None,
            }
        ],
    }
}
```

**メリット:**
- `submit!` の既存インフラをそのまま活用
- proc-macroで完結（コンパイル時に全て解決）
- 段階的な移行が可能（新旧両方が共存可能）

**必要な実装:**
- `gen_function_from_python!` - 関数用proc-macro（→ `PyFunctionInfo`）
- `gen_methods_from_python!` - メソッド用proc-macro（→ `PyMethodsInfo`）
- `#[gen_stub_pyfunction(python = "...")]` - 既存マクロの拡張
- `#[gen_stub_python("...")]` - メソッド用attribute macro（要検討）

### ✅ 最終決定: Python型をそのまま使う

**重要な発見:** Option 1-3は全て間違った前提（Python型をRust型に変換する必要がある）に基づいていました。

**実際には:** `TypeInfo` は単なる文字列とimport情報を保持するだけなので、**Rust型への変換は不要**です！

```rust
// TypeInfo の定義（既存）
pub struct TypeInfo {
    pub name: String,           // Python型文字列をそのまま
    pub import: HashSet<ImportRef>,
}
```

**新しいアプローチ:**

```rust
gen_function_from_python! {
    r#"
        import builtins
        from typing import Optional

        def sum(v: list[int]) -> Optional[int]:
            """Sum integers in a list"""
    "#
}
```

↓ **proc-macroが生成** ↓

```rust
PyFunctionInfo {
    name: "sum",
    args: &[ArgInfo {
        name: "v",
        signature: None,
        r#type: || ::pyo3_stub_gen::TypeInfo {
            name: "list[int]".to_string(),  // ← そのまま
            import: ::std::collections::HashSet::from([
                "builtins".into()
            ])
        },
    }],
    r#return: || ::pyo3_stub_gen::TypeInfo {
        name: "Optional[int]".to_string(),  // ← そのまま
        import: ::std::collections::HashSet::from([
            "builtins".into(),
            "typing".into()
        ])
    },
    doc: "Sum integers in a list",
    module: None,
    is_async: false,
    deprecated: None,
    type_ignored: None,
}
```

**proc-macroが行う処理:**
1. stub文字列をパース
2. `import` 文から必要なモジュールを抽出
3. 関数定義から引数・戻り値の型文字列を抽出
4. docstringを抽出
5. 上記を `PyFunctionInfo` の構造体生成コードに変換

**メリット:**
- ✅ **シンプル** - Python型をそのまま文字列として扱うだけ
- ✅ **Python標準構文** - 本物のPython型アノテーションが使える
- ✅ **型マッピング不要** - `list[int]` → `Vec<???>` の変換が不要
- ✅ **import制御** - ユーザーがimport文で明示的に指定
- ✅ **柔軟** - カスタム型もそのまま使える
- ✅ **責任分離** - 型の正しさはユーザーの責任（`.pyi`を書くのと同じ）

**デメリット:**
- ⚠️ 型の正しさをRustコンパイラが検証できない（ユーザーの責任）
- ⚠️ タイポがあっても気づきにくい

**既存の仕組みとの関係:**
- `PyFunctionInfo` の構造は変更不要
- `TypeInfo` は既にこの用途に対応している
- `ImportRef::from(&str)` も使える
- 既存のコードと共存可能

## 実装計画（更新版）

### Phase 1: 技術調査と設計 ✅

- [x] `gen_stub_pyfunction` の実装を理解
  - `ItemFn` → `PyFunctionInfo` の変換フロー
  - `ArgInfo` の構造（`name`, `TypeOrOverride`）
  - 特殊引数のスキップ処理
- [x] `TypeInfo` の構造を確認
  - `name: String` - Python型文字列をそのまま格納
  - `import: HashSet<ImportRef>` - 必要なimport情報
  - `ImportRef::from(&str)` が利用可能
- [x] 型マッピングの問題を解決
  - Python型をRust型に変換する必要はない
  - Python型文字列をそのまま使う方針に決定
- [x] Pythonパーサーライブラリの選定
  - **決定:** `rustpython-parser` を使用
  - 理由: 型ヒントは `Optional[list[tuple[int, str]]]` や `collections.abc.Callable[[str], typing.Any]` のように複雑になるため、完全なPython ASTパーサーが必要

### Phase 2: プロトタイプ実装（関数のみ）✅

まずは **関数のみ** に焦点を絞る：

**実装の関係性:**

Option C は Option A の糖衣構文として実装：

```rust
// Option C（ユーザーが書くコード）
#[gen_stub_pyfunction(python = r#"def foo(): ..."#)]
#[pyfunction]
fn foo() { ... }

// ↓ 内部でOption Aの機能を使用
```

**実装順序:**
1. ✅ Option A: `gen_function_from_python!` の実装（コア）
2. ✅ Option C: `#[gen_stub_pyfunction(python = "...")]` の実装（ラッパー）

- [x] proc-macro の基本構造
  - `pyo3-stub-gen-derive/src/gen_stub/gen_from_python.rs` (新モジュール)
  - 共通のパーサーとコード生成ロジック

- [x] **Step 1: Option A の実装**（コア機能）
  - `gen_function_from_python!` proc-macro
  - Python stub 文字列を受け取る
  - `PyFunctionInfo` 構造体のトークンストリームを生成
  - `submit!` 内で使用される想定

- [x] **Step 2: Option C の実装**（糖衣構文）
  - 既存の `gen_stub_pyfunction` を拡張
  - `python = "..."` パラメータを受け取る
  - `python`パラメータが指定された場合は自動生成を抑制
  - 内部で `parse_python_function_stub()` を呼び出す

- [x] Python stub パーサー
  - [x] import文の抽出
  - [x] 関数定義のパース（`def func_name(args) -> return: ...`）
  - [x] 引数リストのパース（`name: type` の形式）
  - [x] 戻り値型のパース
  - [x] docstringの抽出

- [x] コード生成
  - [x] `PyFunctionInfo` 構造体のトークンストリーム生成
  - [x] `ArgInfo` の生成（型文字列 → `TypeInfo` クロージャ）
  - [x] import情報の `HashSet<ImportRef>` への変換

- [x] エラーハンドリング
  - [x] パースエラーの適切な報告
  - [x] エラー位置の表示

**パーサーの選択:**

**`rustpython-parser` を使用**
- 完全なPython ASTパーサー（crates.ioに公開済み）
- メリット:
  - 正確なPython構文解析
  - 複雑な型ヒント（`Optional[list[tuple[int, str]]]`, `collections.abc.Callable[[str], typing.Any]` など）に対応
  - 将来の拡張性が高い
  - デフォルト引数、可変長引数、async関数なども対応可能
- デメリット:
  - proc-macroのビルド時間増加の可能性（実用上は問題ないレベルと想定）

**手書きパーサーは採用しない理由:**
- Python型ヒントは非常に複雑になり得る
- ネストした型パラメータ `[[str], typing.Any]` のパースは困難
- 正規表現や簡易パーサーでは保守性が低い

### Phase 3: テストと検証 ✅

- [x] ユニットテスト（`pyo3-stub-gen-derive/src/gen_stub/gen_from_python.rs`）
  - [x] 基本的な関数パース (`test_basic_function`)
  - [x] import文の処理 (`test_function_with_imports`)
  - [x] docstringの抽出 (各テストケースで確認)
  - [x] 複数引数の処理 (`test_multiple_args`)
  - [x] 複雑な型の処理 (`test_complex_types`)
  - [x] 戻り値なしの処理 (`test_no_return_type`)
  - すべてのテストは`insta`スナップショットテストで検証済み

- [x] 統合テスト（`examples/pure`）
  - [x] **Option A のテスト: `fn_with_python_stub`**（`examples/pure/src/overriding.rs`）
    ```rust
    #[pyfunction]
    pub fn fn_with_python_stub<'a>(callback: Bound<'a, PyAny>) -> PyResult<Bound<'a, PyAny>> {
        callback.call1(("World!",))?;
        Ok(callback)
    }

    pyo3_stub_gen::inventory::submit! {
        pyo3_stub_gen::derive::gen_function_from_python! {
            r#"
            import collections.abc
            import typing

            def fn_with_python_stub(callback: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]:
                """Example function using gen_function_from_python! macro."""
            "#
        }
    }
    ```
    - ✅ stub生成確認
    - ✅ pytest (21/21) パス
    - ✅ pyright 型チェックパス
    - ✅ ruff チェックパス

  - [x] **Option C のテスト: `fn_with_python_param`**（`examples/pure/src/overriding.rs`）
    ```rust
    #[gen_stub_pyfunction(python = r#"
        import collections.abc
        import typing

        def fn_with_python_param(callback: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]:
            """Example using python parameter in gen_stub_pyfunction attribute."""
    "#)]
    #[pyfunction]
    pub fn fn_with_python_param<'a>(callback: Bound<'a, PyAny>) -> PyResult<Bound<'a, PyAny>> {
        callback.call1(("Option C!",))?;
        Ok(callback)
    }
    ```
    - ✅ 既存の自動生成と衝突しないことを確認
    - ✅ stub生成確認
    - ✅ pytest パス
    - ✅ pyright 型チェックパス
    - ✅ ruff チェックパス

- [ ] ドキュメント更新
  - [ ] `CLAUDE.md`に新機能を追加
  - [ ] 使用例とベストプラクティス
  - [ ] 既存の`submit!`との使い分けガイド

### Phase 4: メソッドのサポート（要検討）

関数が動作したら、メソッドにも対応：

**Option A: `submit!` approach**
- [ ] `gen_methods_from_python!` マクロの実装
  - `class: StructName` パラメータで対象のクラスを指定
  - メソッドのパース（`self` 引数の処理）
  - `PyMethodsInfo` の生成

**Option C: attribute macro approach（実装の複雑さ次第）**
- [ ] `#[gen_stub_python("...")]` attribute macroの検討
  - 個別メソッドに適用
  - 既存の `#[gen_stub_pymethods]` との統合方法
  - 実装の複雑さを評価

- [ ] `examples/pure` での検証（`examples/pure/src/manual_submit.rs`）
  - [ ] `Incrementer::increment_1` の変換
  - [ ] `Incrementer2` の変換

### Phase 5: 既存コードの移行（完了後）

- [ ] `examples/pure` の `submit!` を変換
  - [ ] `examples/pure/src/overloading.rs`
    - [ ] `overload_example_1`
    - [ ] `overload_example_2`
  - [ ] `examples/pure/src/manual_submit.rs`
    - [ ] `Incrementer::increment_1`
    - [ ] `Incrementer2` の複数メソッド
  - [ ] `examples/pure/src/overriding.rs`
    - [ ] `fn_override_type`

- [ ] 他のexamplesも確認


## 未解決の課題・検討事項

### 1. パーサーの実装方針

**✅ 決定:** `rustpython-parser` を使用

**`rustpython-parser` で対応する構文:**
- [x] 基本: `def func(arg: Type) -> RetType: ...`
- [x] import: `import module`, `from module import name`
- [x] docstring: `"""doc"""`
- [x] 複雑な型: `Optional[list[tuple[int, str]]]`, `collections.abc.Callable[[str], typing.Any]`
- [ ] デフォルト引数: `def func(x: int = 10): ...` （将来対応）
- [ ] 可変長引数: `def func(*args: int, **kwargs: str): ...` （将来対応）
- [ ] async関数: `async def func(): ...` （将来対応）

### 2. エラーメッセージの品質

proc-macro内でパースエラーが起きた場合、どう報告するか：
- エラー位置の特定（行番号、カラム）
- stub文字列内の位置をソースコード位置にマッピング
- 分かりやすいエラーメッセージ

### 3. 型アノテーションの正しさの検証

ユーザーが書いたPython型が正しいかどうかの検証は？
- **現状の方針:** 検証しない（ユーザーの責任）
- **将来的な改善案:**
  - `stubtest` を実行時に自動実行
  - よくあるタイポを警告（`interger` → `int` など）

### 4. 複数の関数定義への対応

stub文字列内に複数の関数を書けるべきか？

```rust
gen_functions_from_python! {  // 複数形
    r#"
        def foo(x: int) -> int: ...
        def bar(y: str) -> str: ...
    "#
}
```

→ 単一のマクロ呼び出しで複数の `PyFunctionInfo` を生成？

### 5. メソッドの場合の構文

```rust
gen_methods_from_python! {
    class: Incrementer,  // ← Rustのクラス（struct）名が必要
    method_stub: r#"
        def increment_1(self, x: int) -> int: ...
    "#
}
```

`class:` パラメータでRustのクラス（struct）を指定する必要がある
（`struct_id: TypeId::of::<Incrementer>` のため）

## 次のステップ

1. ✅ 既存実装の理解（完了）
2. ✅ 設計方針の決定（完了）
3. ✅ パーサーの選択（完了: `rustpython-parser`）
4. ✅ 実装完了
   - `rustpython-parser` を使用してPython stub をパース
   - `PyFunctionInfo` の生成コード実装
   - Option A: `gen_function_from_python!` マクロ
   - Option C: `#[gen_stub_pyfunction(python = "...")]` 属性
5. ✅ `examples/pure` で動作確認
   - すべてのテストがパス
6. ⏳ ドキュメント更新とフィードバック
   - [ ] `CLAUDE.md`に新機能を追加
   - [ ] 既存の`submit!`との使い分けガイド
7. 🔄 次フェーズ（オプション）
   - メソッドへの対応 (Phase 4)
   - オーバーロードの既存コード移行 (Phase 5)

## 参考リンク

- RustPython Parser: https://github.com/RustPython/Parser
- rustpython-parser crate: https://crates.io/crates/rustpython-parser
- Python AST documentation: https://docs.python.org/3/library/ast.html
- syn (Rust parser): https://docs.rs/syn/latest/syn/
