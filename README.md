# rust-protobuf-sample

Rustでprotobufを遊ぶやつ

## protobufのgo_outするときの対応したメモ

出るエラー

```zsh
❯ protoc -I=./ --go_out=./ user.proto
protoc-gen-go: program not found or is not executable
--go_out: protoc-gen-go: Plugin failed with status code 1.
```

```zsh
❯ go get -d -u github.com/golang/protobuf/protoc-gen-go
❯ go install github.com/golang/protobuf/protoc-gen-go
```

## rust-protobufで空ファイルをデコードする

[Golangで出力したprotobufバイナリをRustで読み込む \- 逆さまにした](https://cipepser.hatenablog.com/entry/protobuf-read-in-rust)のところまでは同じ。

`main.rs`の読み込むファイルを`go_user.bin`から`zero.bin`に変更する。

```zsh
❯ touch zero.bin

❯ cargo run
Name:
Age: 0
```

[Language Guide \(proto3\)  \|  Protocol Buffers  \|  Google Developers](https://developers.google.com/protocol-buffers/docs/proto3#unknowns)
に以下のように書いてある。

> Unknown fields are well-formed protocol buffer serialized data representing fields that the parser does not recognize. For example, when an old binary parses data sent by a new binary with new fields, those new fields become unknown fields in the old binary.

protobufのunknow fieldsは定義済みなので、パーサーは値が存在しないのか、フィールドが定義されていないのかは区別できない。

## reservedとは

ところで以下のように書かれているのが気になった。

> Originally, proto3 messages always discarded unknown fields during parsing, but in version 3.5 we reintroduced the preservation of unknown fields to match the proto2 behavior. In versions 3.5 and later, unknown fields are retained during parsing and included in the serialized output.

手元の環境は`3.7.1`なので、`3.5`以降。

```zsh
❯ protoc --version
libprotoc 3.7.1
```

`.proto`を以下のように変更してみる。

```proto
syntax = "proto3";
package user;

message User {
  string name = 1;
  int32 age = 2;
  reserved 3;
}
```

バイナリも以下のように変更。
（`age`をreservedとして使おうとしている）

```zsh
# 前
❯ hexdump go_user.bin
0000000 0a 05 41 6c 69 63 65 10 14 0a
0000009

# 後
❯ hexdump reserved.bin
0000000 0a 05 41 6c 69 63 65 30 14 0a
000000a
```

rustで実行

```zsh
❯ cargo run
   Compiling rust-protobuf-sample v0.1.0 (/Users/cipepser/.go/src/github.com/cipepser/rust-protobuf-sample)
    Finished dev [unoptimized + debuginfo] target(s) in 0.57s
     Running `target/debug/rust-protobuf-sample`
thread 'main' panicked at 'fail to merge: WireError(UnexpectedEof)', src/libcore/result.rs:999:5
note: Run with `RUST_BACKTRACE=1` environment variable to display a backtrace.
```

確かにフィールドそのものが存在しないのでどのvalue typeとして読み込んでいいかわからないはず。
だからreservedなしでも同じだと思う。

ということで`.proto`を戻す。

```proto
syntax = "proto3";
package user;

message User {
  string name = 1;
  int32 age = 2;
}
```

```zsh
❯ protoc --rust_out src/ user.proto

❯ protoc -I=./ --go_out=./ user.proto
❯ go run main.go

❯ hexdump go_user.bin
0000000 0a 05 41 6c 69 63 65 10 14
0000009
```

最後の`0a`の有無が変わるからなんか渡す情報は増えてるのかなぁ。

rustで実行。

```zsh
❯ cargo run
   Compiling rust-protobuf-sample v0.1.0 (/Users/cipepser/.go/src/github.com/cipepser/rust-protobuf-sample)
    Finished dev [unoptimized + debuginfo] target(s) in 0.57s
     Running `target/debug/rust-protobuf-sample`
thread 'main' panicked at 'fail to merge: WireError(UnexpectedEof)', src/libcore/result.rs:999:5
note: Run with `RUST_BACKTRACE=1` environment variable to display a backtrace.
```

同じく`WireError(UnexpectedEof)`になった。

## rust-protobufのdefault

ゼロ値のところの話に戻る。
proto3の仕様からgoでいうゼロ値のようにデフォルト値が使われることがわかった。
rust-protobufではどうなっているのか。

`protobuf-codegen`で生成したメソッドを利用する以下コードの返り値は`Option`型ではない。

```rust
println!("Name: {}", u.get_name());
println!("Age: {}", u.get_age());
```

`protoc --rustout`で吐き出された実装を確認すると以下のようになっていることからも`Option`に包まれていないことがわかる。

```rust
pub fn get_name(&self) -> &str {
    &self.name
}

pub fn get_age(&self) -> i32 {
    self.age
}
```

ちゃんとproto3の仕様に従っている。
ところでGoの場合はゼロ値があるが、Rustではどうやって実現されているんだろうか。

ということでコンストラクタを見てみる。

```rust
impl User {
    pub fn new() -> User {
        ::std::default::Default::default()
    }
}
```

予想通りだけど`default`が使われている。
`User`の定義でも`Default`がderiveされている。

```rust
#[derive(PartialEq,Clone,Default)]
pub struct User {
    // message fields
    pub name: ::std::string::String,
    pub age: i32,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}
```

上記の`::std::default::Default`の実装はこんな感じ。

```rust
impl<'a> ::std::default::Default for &'a User {
    fn default() -> &'a User {
        <User as ::protobuf::Message>::default_instance()
    }
}
```

## References
- [Golangで出力したprotobufバイナリをRustで読み込む \- 逆さまにした](https://cipepser.hatenablog.com/entry/protobuf-read-in-rust)
- [Language Guide \(proto3\)  \|  Protocol Buffers  \|  Google Developers](https://developers.google.com/protocol-buffers/docs/proto3#unknowns)