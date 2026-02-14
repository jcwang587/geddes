# @jcwang587/geddes

Node.js bindings for the `geddes` Rust parser, built with `napi-rs`.

## Local development

```bash
npm install
npm run build
```

## Exported API

- `read(path: string): Pattern`
- `readBytes(data: Buffer, filename: string): Pattern`

`Pattern` shape:

```ts
type Pattern = {
  x: number[]
  y: number[]
  e?: number[]
}
```

## Publishing

This package uses the standard `napi-rs` prebuilt flow:

```bash
npm run build
npm run artifacts
npm run prepublishOnly
```
