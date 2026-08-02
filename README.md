# ipaddr_disp

サイネージ用の自PC IPアドレス表示アプリ（Windows / wgpu + DX12）

## 概要

`eframe` / `egui` で作られたデスクトップアプリです。5 秒ごとに自 PC の IP アドレスを取得し、常に最前面に表示します。デジタルサイネージやリモート接続時の IP 確認用途を想定しています。

## 主な機能

- 自 PC のローカル IP アドレスを自動表示
- 5 秒ごとに自動更新
- 常に最前面表示（`always-on-top`）
- ウィンドウサイズ 400x200 のコンパクト表示
- 手動更新ボタン「今すぐ更新」
- 日本語フォント（Noto Sans JP）同梱

## 使用技術

- Rust（edition 2024）
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) / egui 0.35
- wgpu（バックエンドを DX12 に固定）

## 必要要件

- Rust（最新の安定版）
- Windows（DX12 バックエンドを使用）

## ビルド方法

```bash
cargo build --release
```

## 実行方法

```bash
cargo run --release
```

生成された実行ファイルは `target/release/ipaddr_disp.exe` です。

## 注意事項

- `#![windows_subsystem = "windows"]` はコメントアウトされているため、起動時にコンソールウィンドウが表示されます。コンソールを非表示にしたい場合は `src/main.rs` 先頭のコメントを解除してください。
- IP アドレスは `UdpSocket` を `0.0.0.0:0` にバインドして `8.8.8.8:80` に接続する手法で取得しています（実際に通信は行いません）。

## ライセンス

- 本アプリのコード: 特になし（未指定）
- 同梱フォント: [Noto Sans JP](https://fonts.google.com/noto/specimen/Noto+Sans+JP)（OFL 1.1、`assets/OFL.txt`）
