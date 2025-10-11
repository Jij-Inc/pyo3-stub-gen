# TODO: Python Stub構文からのメタデータ生成

## 背景

現在、型のオーバーロード（`@overload`）を実現するために、`submit!`マクロで手動で`PyFunctionInfo`や`PyMethodsInfo`を構築する必要がある。この方法は冗長で、Python開発者にとって直感的ではない。

### 現在の問題点

```rust
// examples/pure/src/lib.rs:473-498
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
- [ ] Pythonパーサーライブラリの選定
  - 候補: `ruff_python_parser`, `rustpython-parser`, 手書き簡易パーサー
  - 評価基準: メンテナンス状況、パフォーマンス、実装の複雑さ

### Phase 2: プロトタイプ実装（関数のみ）

まずは **関数のみ** に焦点を絞る：

**実装の関係性:**

Option C は Option A の糖衣構文として実装：

```rust
// Option C（ユーザーが書くコード）
#[gen_stub_pyfunction(python = r#"def foo(): ..."#)]
#[pyfunction]
fn foo() { ... }

// ↓ proc-macroが展開

// Option A（展開後のコード）
#[pyfunction]
fn foo() { ... }

inventory::submit! {
    gen_function_from_python! {
        r#"def foo(): ..."#
    }
}
```

**実装順序:**
1. Option A: `gen_function_from_python!` の実装（コア）
2. Option C: `#[gen_stub_pyfunction(python = "...")]` の実装（ラッパー）

- [ ] proc-macro の基本構造
  - `pyo3-stub-gen-derive/src/gen_stub/gen_from_python.rs` (新モジュール)
  - 共通のパーサーとコード生成ロジック

- [ ] **Step 1: Option A の実装**（コア機能）
  - `gen_function_from_python!` proc-macro
  - Python stub 文字列を受け取る
  - `PyFunctionInfo` 構造体のトークンストリームを生成
  - `submit!` 内で使用される想定

- [ ] **Step 2: Option C の実装**（糖衣構文）
  - 既存の `gen_stub_pyfunction` を拡張
  - `python = "..."` パラメータを受け取る
  - 関数定義をそのまま出力 + `inventory::submit!` ブロックを追加
  - 内部で `gen_function_from_python!` を呼び出すだけ

- [ ] Python stub パーサー
  - [ ] import文の抽出
  - [ ] 関数定義のパース（`def func_name(args) -> return: ...`）
  - [ ] 引数リストのパース（`name: type` の形式）
  - [ ] 戻り値型のパース
  - [ ] docstringの抽出

- [ ] コード生成
  - [ ] `PyFunctionInfo` 構造体のトークンストリーム生成
  - [ ] `ArgInfo` の生成（型文字列 → `TypeInfo` クロージャ）
  - [ ] import情報の `HashSet<ImportRef>` への変換

- [ ] エラーハンドリング
  - [ ] パースエラーの適切な報告
  - [ ] エラー位置の表示

**パーサーの選択肢:**

**Option A: 軽量手書きパーサー（推奨）**
- 対象: 関数定義とimport文のみ
- 必要な構文:
  ```python
  import module
  from module import name
  def func_name(arg1: Type1, arg2: Type2) -> RetType:
      """docstring"""
  ```
- 実装: 正規表現 or `nom` などの軽量パーサーコンビネータ
- メリット: 依存が少ない、シンプル、必要最小限
- デメリット: 複雑なPython構文は非対応

**Option B: `ruff_python_parser`**
- 完全なPython ASTパーサー
- メリット: 正確、将来の拡張性
- デメリット: 重い依存、proc-macroのビルド時間増加

**初期プロトタイプはOption Aで開始し、必要に応じてOption Bに移行**

### Phase 3: テストと検証

- [ ] ユニットテスト（`pyo3-stub-gen-derive/tests/`）
  - [ ] 基本的な関数パース
    ```rust
    gen_function_from_python! {
        r#"def foo(x: int) -> int: ..."#
    }
    ```
  - [ ] import文の処理
    ```rust
    gen_function_from_python! {
        r#"
            import builtins
            from typing import Optional
            def foo(x: Optional[int]) -> int: ...
        "#
    }
    ```
  - [ ] docstringの抽出
  - [ ] 複数引数の処理
  - [ ] パースエラーのテスト

- [ ] 統合テスト（`examples/pure`）
  - [ ] **最初のテストケース: `fn_override_type`**

    **Step 1: Option A で直接テスト**
    ```rust
    #[pyfunction]
    fn fn_override_type<'a>(
        cb: Bound<'a, PyAny>,
    ) -> PyResult<Bound<'a, PyAny>> {
        cb.call1(("Hello!",))?;
        Ok(cb)
    }

    submit! {
        gen_function_from_python! {
            r#"
                import collections.abc
                import typing

                def fn_override_type(cb: collections.abc.Callable[[str], typing.Any]) -> collections.abc.Callable[[str], typing.Any]: ...
            "#
        }
    }
    ```
    - 既存の `#[gen_stub_pyfunction]` と `#[gen_stub(override_type)]` を削除
    - `cargo run --bin stub_gen` でstub生成
    - 生成された`.pyi`が既存と同じか確認
    - `task pure:test` で型チェックが通ることを確認

    **Step 2: Option C で書き直し**（Option A が動作したら）
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
    - 既存の `submit!` ブロックを削除
    - より簡潔に書けることを確認

  - [ ] **オーバーロードのテスト: `overload_example_1`**（Option A を使用）
    ```rust
    #[gen_stub_pyfunction]
    #[pyfunction]
    fn overload_example_1(x: f64) -> f64 { x + 1.0 }

    // 既存のsubmit!を置き換え
    submit! {
        gen_function_from_python! {
            r#"
                import builtins
                def overload_example_1(x: int) -> int: ...
            "#
        }
    }
    ```
    - `@overload`が正しく出力されるか確認

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

- [ ] `examples/pure` での検証
  - [ ] `Incrementer::increment_1` の変換
  - [ ] `Incrementer2` の変換

### Phase 5: 既存コードの移行（完了後）

- [ ] `examples/pure/src/lib.rs`の`submit!`を変換
  - [ ] `overload_example_1` (lines 391-406)
  - [ ] `overload_example_2` (lines 420-452)
  - [ ] `Incrementer` (lines 473-498)
  - [ ] `Incrementer2` (lines 518-569)

- [ ] 他のexamplesも確認


## 未解決の課題・検討事項

### 1. パーサーの実装方針

**決定待ち:** 軽量手書きパーサー vs `ruff_python_parser`

**手書きパーサーで対応が必要な構文:**
- [x] 基本: `def func(arg: Type) -> RetType: ...`
- [x] import: `import module`, `from module import name`
- [x] docstring: `"""doc"""`
- [ ] 複雑な型: `Optional[list[tuple[int, str]]]`
- [ ] デフォルト引数: `def func(x: int = 10): ...`
- [ ] 可変長引数: `def func(*args: int, **kwargs: str): ...`
- [ ] async関数: `async def func(): ...`

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
3. ⏳ パーサーの選択と実装開始
   - まず軽量手書きパーサーでプロトタイプ
   - 基本的な関数定義のみサポート
4. ⏳ `examples/pure` で動作確認
5. ⏳ フィードバックを得て改善

## 参考リンク

- RustPython Parser: https://github.com/RustPython/Parser
- Ruff: https://github.com/astral-sh/ruff
- Python AST documentation: https://docs.python.org/3/library/ast.html
- nom parser combinator: https://github.com/rust-bakery/nom
- syn (Rust parser): https://docs.rs/syn/latest/syn/
