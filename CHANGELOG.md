# Changelog

## [0.1.8](https://github.com/Roberdan/convergio-inference/compare/v0.1.7...v0.1.8) (2026-04-18)


### Bug Fixes

* **deps:** bump rustls-webpki for RUSTSEC-2026-0099 ([885c2ad](https://github.com/Roberdan/convergio-inference/commit/885c2ad79c985b53b639caf68d317e83a7fde16f))
* security and quality audit pass 2 ([e4bdf0f](https://github.com/Roberdan/convergio-inference/commit/e4bdf0ff76c548fdf910dbb49bbde397f11409ad))
* security audit pass  input validation, info leak, infinite loop guard2 ([7b4888a](https://github.com/Roberdan/convergio-inference/commit/7b4888a78c882bfaed5e3e127409f4f5af51726e))

## [0.1.7](https://github.com/Roberdan/convergio-inference/compare/v0.1.6...v0.1.7) (2026-04-14)


### Features

* model override + Qwen/Copilot endpoints ([7dd4578](https://github.com/Roberdan/convergio-inference/commit/7dd457868c9ca605b71b6ef59cf1d5b28c75e99a))

## [0.1.6](https://github.com/Roberdan/convergio-inference/compare/v0.1.5...v0.1.6) (2026-04-13)


### Bug Fixes

* pass CARGO_REGISTRY_TOKEN to release workflow ([70fc970](https://github.com/Roberdan/convergio-inference/commit/70fc970a73fc3e997ed0a20caa632e2836b1ba0e))

## [0.1.5](https://github.com/Roberdan/convergio-inference/compare/v0.1.4...v0.1.5) (2026-04-13)


### Bug Fixes

* add crates.io publishing metadata (description, repository) ([91136e9](https://github.com/Roberdan/convergio-inference/commit/91136e96449b193a4daeeb4a9afbb420d1608da4))

## [0.1.4](https://github.com/Roberdan/convergio-inference/compare/v0.1.3...v0.1.4) (2026-04-13)


### Features

* adapt convergio-inference for standalone repo ([d5cadef](https://github.com/Roberdan/convergio-inference/commit/d5cadef206b4ab5a6ed8fe037333569b1ae7c66d))


### Bug Fixes

* align SDK dependency to v0.1.9 for type compatibility ([15ae815](https://github.com/Roberdan/convergio-inference/commit/15ae8150e82dc261e9fe370c93f837af78f338db))
* **release:** use vX.Y.Z tag format (remove component) ([4d815fb](https://github.com/Roberdan/convergio-inference/commit/4d815fb274ba3efb919f0c3d9ff1032738f23105))
* security audit — MLX injection, input limits, SSRF, SQL hardening ([#3](https://github.com/Roberdan/convergio-inference/issues/3)) ([3435efd](https://github.com/Roberdan/convergio-inference/commit/3435efdadc2b4f92532fdac7672e44ec692c4ffd))


### Documentation

* add .env.example with required environment variables ([#4](https://github.com/Roberdan/convergio-inference/issues/4)) ([851f5d9](https://github.com/Roberdan/convergio-inference/commit/851f5d99ece7c811f61163803c369e2a182f5d7f))
* copy ADR from monorepo ([1374f8f](https://github.com/Roberdan/convergio-inference/commit/1374f8fd4bc759927b5bb84c110690f25450237f))

## [0.1.3](https://github.com/Roberdan/convergio-inference/compare/convergio-inference-v0.1.2...convergio-inference-v0.1.3) (2026-04-12)


### Bug Fixes

* align SDK dependency to v0.1.9 for type compatibility ([15ae815](https://github.com/Roberdan/convergio-inference/commit/15ae8150e82dc261e9fe370c93f837af78f338db))

## [0.1.2](https://github.com/Roberdan/convergio-inference/compare/convergio-inference-v0.1.1...convergio-inference-v0.1.2) (2026-04-12)


### Documentation

* add .env.example with required environment variables ([#4](https://github.com/Roberdan/convergio-inference/issues/4)) ([851f5d9](https://github.com/Roberdan/convergio-inference/commit/851f5d99ece7c811f61163803c369e2a182f5d7f))

## [0.1.1](https://github.com/Roberdan/convergio-inference/compare/convergio-inference-v0.1.0...convergio-inference-v0.1.1) (2026-04-12)


### Features

* adapt convergio-inference for standalone repo ([d5cadef](https://github.com/Roberdan/convergio-inference/commit/d5cadef206b4ab5a6ed8fe037333569b1ae7c66d))


### Bug Fixes

* security audit — MLX injection, input limits, SSRF, SQL hardening ([#3](https://github.com/Roberdan/convergio-inference/issues/3)) ([3435efd](https://github.com/Roberdan/convergio-inference/commit/3435efdadc2b4f92532fdac7672e44ec692c4ffd))


### Documentation

* copy ADR from monorepo ([1374f8f](https://github.com/Roberdan/convergio-inference/commit/1374f8fd4bc759927b5bb84c110690f25450237f))

## 0.1.0 (Initial Release)

### Features

- Initial extraction from convergio monorepo
