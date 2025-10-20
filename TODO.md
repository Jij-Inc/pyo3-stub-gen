# Parameter モデル再設計メモ

## 背景と課題
- 現状の `FunctionDef` / `MethodDef` は `Vec<Arg>` で引数を保持しており、Python のシグネチャ構文（位置限定 `/`、キーワード限定 `*`、可変長、デフォルト値の混在など）を十分に表現できない。
- `Arg` という名前は Python 公式ドキュメントの用語とずれており、読み手に混乱を与える。Python では関数定義の項目は **Parameters** と呼ばれるため、新設計では `Vec<Arg>` に代わるセクション付きコンテナ `Parameters` と、各要素としての `Parameter` 型を導入する。
- 区切り記号 (`/`, `*`) を疑似的な引数として扱っているため、生成フェーズでの整合性検証・整形ロジックが複雑化している。
- PyO3 側の `#[pyo3(signature = ...)]` から得られる情報を十分に活用できておらず、keyword-only 指定やデフォルト値が欠落したままになるケースがある。
- `gen_function_from_python!` / `gen_methods_from_python!` による Python stub 上書き（`parse_python` モジュール）が、Rust 側を経由せずに直接メタデータを生成するため、この経路でも `Parameters` 情報を表現できるよう `type_info` レベルの構造を刷新する必要がある。

## ゴール
- Python 3 の関数定義構文に忠実なパラメータモデルを提供し、位置限定・キーワード限定・可変長・デフォルト値・型情報を正しく保持できるようにする。
- `Arg` 系の型を `Parameters`（全体コンテナ）・`Parameter`（個別パラメータ）へ改名し、API とドキュメントを Python 用語と揃える。
- 生成される `.pyi` が CPython 互換のシグネチャを持ち、手動で書く場合でも扱いやすい API にする。
- 従来 `SignatureArg` が担っていた責務を新モデルへ統合し、余剰な中間型を排除する。
- Rust の関数定義から得られる型ヒントと `#[pyo3(signature = ...)]` が提供する引数構造を矛盾なく統合し、双方を真の情報源として扱う。

## 用語・設計方針
- `Parameter` は「名前」「型 `TypeInfo`」「デフォルト値（任意）」「引数種別」を持つ構造体に再設計する。
  - `ParameterKind`（仮称）で以下を明確に区別する：
    - PositionalOnly (`/` 手前)
    - PositionalOrKeyword
    - KeywordOnly（`*` 以降）
    - VarPositional (`*args`)
    - VarKeyword (`**kwargs`)
- 区切り記号 `/` と `*` は `Parameters` がセクション情報として保持し、疑似的なエントリを生成しない。
- `Parameters` 型は単なる `Vec<Parameter>` ではなく、`positional_only`, `positional_or_keyword`, `keyword_only`, `varargs`, `varkw` のようなセクション構造で保持し、Python の順序規則を型レベルで表現する（セクション間の遷移を明示しておく）。
- 既存の `SignatureArg` の責務を `Parameters` / `Parameter` へ統合し、`SignatureArg` は完全に廃止する。
- `type_info` 層では `ParameterInfo`（仮称）を定義し、`fn() -> TypeInfo` やデフォルト値生成関数などを保持したまま `inventory` に登録する。`generate` 層で `ParameterInfo` から `Parameter` へ変換して最終的な `Parameters` を構築する。

## PyO3 の `signature` / `text_signature` の現状と方針
### 現状の扱い
- `#[pyo3(signature = (...))]` は `pyo3-stub-gen-derive` の `gen_stub/signature.rs` でパースされ、`ArgInfo.signature: Option<SignatureArg>` に格納される。`SignatureArg` は識別子・デフォルト値・`*`/`**` を粗く区別する enum であり、区切り記号は実質的に「名前のない `Arg`」として `Vec<Arg>` に押し込められている。
- `signature` 属性を指定しない場合は、Rust 側の関数引数から生成した `ArgInfo` がそのまま使われるため、位置限定やキーワード限定は表現できない。
- `signature` は Python のシグネチャ構文だけを表現し、型ヒントは含まないため、Rust 側の `TypeInfo` との統合が不可欠。
- `#[pyo3(text_signature = "(...)")]` は PyO3 が Python 側のドキュメント表示用に付加するものであり、現状パースしておらず挙動にも影響しない。

### 変更後に目指す姿
- `signature` 情報は新しい `Parameters` へ直接マッピングされるようにし、位置限定・キーワード限定・可変長・デフォルト値を構造化して保持する（`SignatureArg` は完全廃止）。
- `text_signature` は従来通り解析対象にせず、PyO3 側の見た目調整に任せる。スタブ生成は `Parameters` に統合した情報のみを信頼する。
- Rust 側の型情報（`TypeInfo`）と `signature` の構造情報を組み合わせたものをソース・オブ・トゥルースとし、スタブ生成ロジックが両者を矛盾なく統合できるようにする。
- 実装後は、Rust の定義＋`signature` から構築した `Parameters` と生成される `.pyi` が一致することをテストで保証する。

## TODO
- [ ] 対応すべきシグネチャパターンを列挙し、テストケース候補として記録する（例：純粋な位置限定、`*, kw` のみ、`/` と `*` の併用、デフォルト・型指定の混在、async 関数）。
- [ ] 典型的なケースごとに、(1) Rust の `#[pyfunction]` / `#[pymethods]` 定義例、(2) `pyo3_stub_gen_derive` が生成すべき `ParameterInfo` 初期化コード、(3) `pyo3_stub_gen::generate` が `Parameters` へ変換し `.pyi` を出力する手順を具体例で整理する。
- [ ] `pyo3-stub-gen/src/generate` 以下のデータ構造を `Parameters` ベースへリファクタリングする。
  - [ ] `Arg` → `Parameter` へのリネームとフィールド再設計。
  - [ ] `FunctionDef` / `MethodDef` に `Parameters` 型を導入し、既存の `Vec<Arg>` を置き換える。
  - [ ] 単なる `Vec` ではなく、`Parameters { positional_only, positional_or_keyword, keyword_only, varargs, varkw }` のような構造体を定義し、区切り記号・順序制約・セクション遷移を厳密に管理する。
  - [ ] 出力時に `/`・`*` を挿入するロジックを新しいセクション構造に基づいて再実装する。
- [ ] `ArgInfo` や `SignatureArg` などメタデータ層の型を `ParameterInfo` 系へ改名・再設計し、`SignatureArg` を廃止して `Parameter` が必要情報を保持するよう更新する。
  - [ ] `ParameterInfo` から `Parameter` への変換ロジックを実装し、`generate` 層で `Parameters` を構築する共通処理を整備する。
- [ ] `pyo3-stub-gen-derive` クレートのパーサ群（`gen_stub/signature.rs`, `arg.rs`, `pymethods.rs`, `pyfunction.rs`）を新モデルに対応させ、`#[pyo3(signature = ...)]` から `/`・`*`・デフォルト値を正しく復元しつつ `Parameters` / `Parameter` を直接構築する。
- [ ] Python stub オーバーライド経路（`parse_python` モジュール）で Rust を介さずに得た引数情報から `Parameters` / `Parameter` を生成できるようにする。
- [ ] `#[pyo3(text_signature = ...)]` を解析対象外とする旨をコードコメントかドキュメントで明記する。
- [ ] `.pyi` 出力処理（`generate/function.rs`, `generate/method.rs`, その他ヘルパー）を `Parameters` から文字列化する形で書き換え、`.pyi` の整形ルールを整理する。
- [ ] 手動構築パス（`class.rs`, `variant_methods.rs` 等）で新 API を利用するヘルパーを追加し、後方互換性を確認する。
- [ ] インポート集計ロジックを見直し、区切り記号が影響しないことを保証する。
- [ ] 多様なシグネチャをカバーするユニットテスト／スナップショットテストを追加し、生成結果を検証する。
- [ ] リファクタリング後の API と設計の背景をドキュメント化する（README、`generate` モジュールの doc コメントなど）。

## 具体例メモ（テストケース候補）

以下では、将来的に `ParameterInfo` / `Parameter` を使った end-to-end テストへ落とし込みたい典型シナリオを正確なデータフローとともに記述する。例に登場する型は以下を想定している。

```rust
pub struct ParameterInfo {
    pub name: &'static str,
    pub kind: ParameterKind,
    pub type_info: fn() -> TypeInfo,
    pub default: ParameterDefault,
}

pub enum ParameterDefault {
    None,
    Expr(fn() -> String),
}

pub enum ParameterKind {
    PositionalOnly,
    PositionalOrKeyword,
    KeywordOnly,
    VarPositional,
    VarKeyword,
}

pub struct Parameter {
    pub name: &'static str,
    pub kind: ParameterKind,
    pub type_info: TypeInfo,
    pub default: Option<String>,
}

/// Python シグネチャのセクション単位で管理するコンテナ
pub struct Parameters {
    pub positional_only: Vec<Parameter>,
    pub positional_or_keyword: Vec<Parameter>,
    pub keyword_only: Vec<Parameter>,
    pub varargs: Option<Parameter>,
    pub varkw: Option<Parameter>,
}

impl Parameters {
    pub fn from_infos(infos: &[ParameterInfo]) -> Self { /* ParameterKind に従って分類 */ }
    pub fn iter_entries(&self) -> impl Iterator<Item = &Parameter> { /* 各セクションを連結して走査 */ }
}
```

### ケースA: 位置引数のみの `#[pyfunction]`

1. **Rust 側定義（入力）**

    ```rust
    #[pyfunction]
    fn add(x: i32, y: i32) -> i32 {
        x + y
    }
    ```

2. **`pyo3_stub_gen_derive` が生成する `PyFunctionInfo`（重要部分のみ）**

    ```rust
::pyo3_stub_gen::type_info::PyFunctionInfo {
    name: "add",
    parameters: &[
        ParameterInfo {
            name: "x",
            kind: ParameterKind::PositionalOrKeyword,
            type_info: || <i32 as PyStubType>::type_input(), // => TypeInfo { name: "builtins.int", import: {"builtins"} }
            default: ParameterDefault::None,
        },
        ParameterInfo {
            name: "y",
            kind: ParameterKind::PositionalOrKeyword,
            type_info: || <i32 as PyStubType>::type_input(),
            default: ParameterDefault::None,
        },
    ],
    r#return: || <i32 as PyStubType>::type_output(),
    doc: "",
    module: None,
    is_async: false,
    deprecated: None,
    type_ignored: None,
    sig: None,
}
    ```

3. **`generate` フェーズでの期待される処理**

```rust
let parameters = Parameters::from_infos(py_function_info.parameters);
let function_def = FunctionDef {
    name: py_function_info.name,
    parameters,
    return_type: (py_function_info.r#return)(),
    doc: py_function_info.doc,
    is_async: py_function_info.is_async,
    deprecated: py_function_info.deprecated.clone(),
    type_ignored: py_function_info.type_ignored,
};
let rendered = function_def.to_string();
assert!(rendered.contains("def add(x: builtins.int, y: builtins.int) -> builtins.int:"));
```

### ケースB: `/` を含む positional-only + keyword-only + デフォルト

1. **Rust 側定義**

    ```rust
    #[pyfunction(signature = (token, /, *, retries = 3, timeout = None))]
    fn send(token: &str, retries: usize, timeout: Option<f64>) -> bool {
        // 実装省略
        true
    }
    ```

2. **派生マクロ出力（抜粋）**

    ```rust
PyFunctionInfo {
    name: "send",
    parameters: &[
        ParameterInfo {
            name: "token",
            kind: ParameterKind::PositionalOnly,
            type_info: || <&str as PyStubType>::type_input(),
            default: ParameterDefault::None,
        },
        ParameterInfo {
            name: "retries",
            kind: ParameterKind::KeywordOnly,
            type_info: || <usize as PyStubType>::type_input(),
            default: ParameterDefault::Expr(|| "3".to_string()),
        },
        ParameterInfo {
            name: "timeout",
            kind: ParameterKind::KeywordOnly,
            type_info: || <Option<f64> as PyStubType>::type_input(),
            default: ParameterDefault::Expr(|| "None".to_string()),
        },
    ],
    r#return: || <bool as PyStubType>::type_output(),
    doc: "",
    module: None,
    is_async: false,
    deprecated: None,
    type_ignored: None,
    sig: Some(/* signature metadata */),
}
    ```

3. **`generate` での期待挙動と最終出力**

```rust
let parameters = Parameters::from_infos(py_function_info.parameters);
let function_def = FunctionDef {
    name: py_function_info.name,
    parameters,
    return_type: (py_function_info.r#return)(),
    doc: py_function_info.doc,
    is_async: py_function_info.is_async,
    deprecated: py_function_info.deprecated.clone(),
    type_ignored: py_function_info.type_ignored,
};
let rendered = function_def.to_string();
assert!(rendered.contains("def send(token: builtins.str, /, *, retries: builtins.int = 3, timeout: typing.Optional[builtins.float] = None) -> builtins.bool:"));
```

    - `Parameters` は最初の positional-only ブロックを認識して `/` を挿入。
    - keyword-only が存在するため `*` も挿入。
    - デフォルト値は `ParameterDefault::Expr` から得た文字列を利用。

### ケースC: 可変長 `*args` / `**kwargs` を含むメソッド (`#[pymethods]`)

1. **Rust 側定義**

    ```rust
    #[pymethods]
    impl Logger {
        #[pyo3(signature = (*messages, **kw))]
        fn log(&self, *messages: &str, **kw: &PyAny) -> None {}
    }
    ```

2. **`pyo3_stub_gen_derive` 出力（メソッド情報）**

    ```rust
MethodInfo {
    name: "log",
    parameters: &[
        ParameterInfo {
            name: "messages",
            kind: ParameterKind::VarPositional,
            type_info: || <&str as PyStubType>::type_input(),
            default: ParameterDefault::None,
        },
        ParameterInfo {
            name: "kw",
            kind: ParameterKind::VarKeyword,
            type_info: || <&PyAny as PyStubType>::type_input(),
            default: ParameterDefault::None,
        },
    ],
    r#return: || <() as PyStubType>::type_output(),
    doc: "",
    r#type: MethodType::Instance,
    is_async: false,
    deprecated: None,
    type_ignored: None,
    sig: Some(/* signature metadata */),
}
    ```

3. **`generate` 側の処理**

    ```rust
    let parameters = Parameters::from_infos(method_info.parameters);
    let method_def = MethodDef {
        name: method_info.name,
        parameters,
        return_type: (method_info.r#return)(),
        doc: method_info.doc,
        r#type: method_info.r#type,
        is_async: method_info.is_async,
        deprecated: method_info.deprecated.clone(),
        type_ignored: method_info.type_ignored,
    };
    let rendered = method_def.to_string();
    assert!(rendered.contains("def log(self, *messages: builtins.str, **kw: typing.Any) -> None:"));
    ```

### ケースD: `parse_python` による stub 上書き（位置限定 + `...` デフォルト）

1. **Python stub 入力**

    ```python
    def parse(data: bytes, /, *, strict: bool = ..., limit: typing.Optional[int] = None) -> Result:
        ...
    ```

2. **`parse_python` の内部で作られる `PyFunctionInfo`**

    ```rust
    PyFunctionInfo {
        name: "parse",
        parameters: &[
            ParameterInfo {
                name: "data",
                kind: ParameterKind::PositionalOnly,
                type_info: || TypeInfo {
                    name: "bytes".to_string(),
                    import: HashSet::new(),
                },
                default: ParameterDefault::None,
            },
            ParameterInfo {
                name: "strict",
                kind: ParameterKind::KeywordOnly,
                type_info: || TypeInfo {
                    name: "bool".to_string(),
                    import: HashSet::new(),
                },
                default: ParameterDefault::Expr(|| "...".to_string()),
            },
            ParameterInfo {
                name: "limit",
                kind: ParameterKind::KeywordOnly,
                type_info: || {
                    // Python stub側からの文字列を保持するため、TypeOrOverride::OverrideType で
                    // "typing.Optional[int]" が渡される想定。
                    TypeInfo {
                        name: "typing.Optional[int]".to_string(),
                        import: HashSet::from([ImportRef::Module("typing".into())]),
                    }
                },
                default: ParameterDefault::Expr(|| "None".to_string()),
            },
        ],
        // 戻り値には TypeOrOverride::OverrideType で "Result" が入る想定
    }
    ```

3. **`generate` の期待出力**

```python
def parse(data: bytes, /, *, strict: bool = ..., limit: typing.Optional[int] = None) -> Result: ...
```

    - `Parameters` が `/` を挿入。
    - `...` というデフォルト値文字列をそのまま出力。

### ケースE: デフォルト値付き `#[pymethods]` インスタンスメソッド（self を含む）

1. **Rust 定義**

    ```rust
    #[pymethods]
    impl Counter {
        #[pyo3(signature = (step = 1))]
        fn incr(&mut self, step: i32) -> i32 {
            self.value += step;
            self.value
        }
    }
    ```

2. **派生マクロ出力の想定（MethodInfo）**

    ```rust
    MethodInfo {
        name: "incr",
        parameters: &[
            ParameterInfo {
                name: "step",
                kind: ParameterKind::PositionalOrKeyword,
                type_info: || <i32 as PyStubType>::type_input(),
                default: ParameterDefault::Expr(|| "1".to_string()),
            },
        ],
        r#return: || <i32 as PyStubType>::type_output(),
        doc: "",
        r#type: MethodType::Instance,
        is_async: false,
        deprecated: None,
        type_ignored: None,
        sig: Some(/* signature metadata */),
    }
    ```

3. **`generate` 出力（`self` は `MethodType::Instance` から付与）**

```rust
let parameters = Parameters::from_infos(method_info.parameters);
let method_def = MethodDef {
    name: method_info.name,
    parameters,
    return_type: (method_info.r#return)(),
    doc: method_info.doc,
    r#type: method_info.r#type,
    is_async: method_info.is_async,
    deprecated: method_info.deprecated.clone(),
    type_ignored: method_info.type_ignored,
};
let rendered = method_def.to_string();
assert!(rendered.contains("def incr(self, step: builtins.int = 1) -> builtins.int:"));
```

- `Parameters` には `step` のみが含まれる。
- `MethodDef::fmt`（または更新されたロジック）が `self` を自動追加。

---

## 実装進捗状況（2025-10-20）

### ✅ 完了したタスク

#### 1. 基盤となる型定義の実装
- **コミット**: `8843ac1` - "Add Parameter model foundation for Python signature syntax"
- `pyo3-stub-gen/src/type_info.rs`:
  - `ParameterKind` enum を追加（PositionalOnly, PositionalOrKeyword, KeywordOnly, VarPositional, VarKeyword）
  - `ParameterDefault` enum を追加（None, Expr(fn() -> String)）
  - `ParameterInfo` struct を追加（コンパイル時メタデータ）
- `pyo3-stub-gen/src/generate/parameters.rs` を新規作成:
  - `Parameter` struct（ランタイム表現）
  - `Parameters` struct（セクション別管理: positional_only, positional_or_keyword, keyword_only, varargs, varkw）
  - `Display` trait 実装（Python スタブ構文生成）
  - 包括的なテスト（4パターン）
- 既存の `Arg`/`ArgInfo`/`SignatureArg` は互換性のため残存

#### 2. generate 層での Parameters モデル移行
- **コミット**: `53d5da2` - "Migrate FunctionDef and MethodDef to use Parameters model"
- `FunctionDef` と `MethodDef` を `Parameters` 使用に変更:
  - `args: Vec<Arg>` → `parameters: Parameters`
  - `Display` 実装を更新
  - `Import` 実装を更新
- `Parameters::from_arg_infos()` 実装（既存 ArgInfo からの変換サポート）
- `Arg` struct と `arg.rs` モジュールを完全削除
- `class.rs`, `variant_methods.rs` の全メソッド生成コードを更新
- `lib.rs` の doctest サンプルを更新
- **テスト結果**: 全25ユニットテスト + 20統合テスト（doctest含む）パス

#### 3. procedural macro での ParameterInfo 生成
- **コミット**: `b810ac6` - "Update procedural macros to generate ParameterInfo instead of ArgInfo"
- `pyo3-stub-gen-derive/src/gen_stub/signature.rs`:
  - `ParameterInfo` を生成するように全面書き換え
  - `/` (positional-only) デリミタのパース対応を追加
  - `SignatureArg::Slash` variant を追加
  - デリミタと位置に基づいて `ParameterKind` を決定
  - `ArgsWithSignature::to_tokens()` を完全に再実装
- `pyfunction.rs`, `method.rs`: `parameters` フィールドを使用
- `type_info.rs`:
  - `PyFunctionInfo.args` → `parameters: &'static [ParameterInfo]`
  - `MethodInfo.args` → `parameters: &'static [ParameterInfo]`
  - `VariantInfo.constr_args` → `&'static [ParameterInfo]`
- `generate/function.rs`, `generate/method.rs`, `generate/variant_methods.rs`:
  - `Parameters::from_infos()` を使用（`from_arg_infos()` から移行）
- **テスト結果**: 全25ユニットテスト + 20統合テスト パス

### 🚧 残タスク

#### 4. variant.rs の更新（complex enum 用）
- [ ] `pyo3-stub-gen-derive/src/gen_stub/variant.rs` の更新
  - `constr_args` の生成を `ParameterInfo` ベースに変更
  - `ArgsWithSignature` の使用を確認・更新

#### 5. parse_python モジュールの更新
- [ ] `pyo3-stub-gen-derive/src/gen_stub/parse_python/pyfunction.rs` の更新
  - Python stub 文字列から `ParameterInfo` を生成
  - 位置限定 (`/`)、キーワード限定 (`*`) のパース対応
- [ ] `pyo3-stub-gen-derive/src/gen_stub/parse_python/pymethods.rs` の更新
  - Python class 定義から `ParameterInfo` を生成
  - メソッドシグネチャのパース対応

#### 6. 旧型の完全削除
- [ ] `pyo3-stub-gen/src/type_info.rs` から `ArgInfo` を削除
- [ ] `pyo3-stub-gen/src/type_info.rs` から `SignatureArg` を削除
- [ ] `pyo3-stub-gen/src/generate/parameters.rs` から `from_arg_infos()` を削除（もしくは deprecated マーク）
- [ ] 全ての参照箇所を確認

#### 7. 統合テストと検証
- [ ] `task stub-gen` を実行して全 example のスタブファイルを生成
- [ ] 生成された `.pyi` ファイルの内容を確認
  - 位置限定パラメータ (`/`) が正しく出力されているか
  - キーワード限定パラメータ (`*`) が正しく出力されているか
  - デフォルト値が正しく出力されているか
- [ ] `task test` を実行して全 example のテストを実行
  - pytest パス確認
  - pyright パス確認
  - ruff パス確認
  - mypy パス確認
  - stubtest パス確認

### 📝 備考

- 現在の実装では、`ArgInfo` と `SignatureArg` が `type_info.rs` に残っているが、これらはもはや使用されていない
- `Parameters::from_arg_infos()` は過渡期の実装で、最終的には削除予定
- parse_python モジュールの更新が完了すれば、完全に新モデルへ移行可能
