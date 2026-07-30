# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## [1.5.0](https://github.com/ManotLuijiu/auto-affiliate-agents/compare/v1.2.4...v1.5.0) (2026-07-30)


### Features

* add mirror mode badge (scrcpy/ADB indicator) ([ded2dc4](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ded2dc438a2227fbed05588139b6c4e018f0c42a))
* add Mirror Relay client support ([0623a09](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0623a092bb482721371b20e2de7c8eda5b56bfbe))
* add Mirror Relay toggle to Settings ([d977516](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/d97751612bd0a04033b7dbacfba8d0475b8b4197))
* add user badge in header showing logged-in email ([7b7ab5a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7b7ab5ae48c6f8642ff1d9fe47eb85134faffe99))
* bundle scrcpy-server and improve path resolution ([2ff9e5d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2ff9e5dcec0f59cf22466d8e896715b3bc4d114f))
* enable real video streaming with screenshot fallback ([297ad28](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/297ad280a9a3361c788ebc04cabebaf1eb4c42f7))
* improve scrcpy UX, secure-screen notice, device icon, friendly names ([2573234](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/25732344b36ab43f0e2f7093886d0af17d62383f))
* **mirror:** add scrcpy-server streaming to #mirror-screen div ([f828603](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f82860333a55525803e61e60ee692eada21b2baf))
* use Google brand PNG logo and update video stream handling ([09f37e6](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/09f37e6e55859ce16390bebb42fa9e575f8d4148))
* use standard-version bumpFiles for multi-file version sync ([9bfeadb](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9bfeadb77b4a845c1400a4d691bf3e773defdbe5))


### Bug Fixes

* add rename_all = snake_case to device_swipe for Tauri arg mapping ([ede1a56](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ede1a56450f254db634641b6c15a540e413164c7))
* add versionrc to sync all version files ([fcae43e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/fcae43e76c1c64eecbe97f55c9194efa1c5050c8))
* apply user changes ([111a884](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/111a88445ec6acbef5c780810e71ef4b092cc5a8))
* implement scrcpy 4.x wire protocol correctly ([515437a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/515437adb66c6508258340b721abff486b07df10))
* improve scrcpy-server process detection ([8ec2b53](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/8ec2b53d6cca1d0a633162a22944eb646cbbcab5))
* **mirror:** fix two critical root causes of live mirror timeout ([455eada](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/455eadac00d76cc900350aace01949a95c295f3b))
* **mirror:** toggle scrcpy now uses scrcpy-server mode ([5fb9811](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/5fb9811cbe156bdc607db07be2a05dbcdd4ca3ff))
* **mirror:** video stream false success state - NAL parsing + first-frame timeout ([dccd3e2](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/dccd3e289fb712708274ea1e0de9777eeb755d13))
* remove nohup, fix device-meta parser for scrcpy 4.x protocol ([1975b9f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/1975b9f592e4ce8303263913731d007d33f22ee8))
* remove TCP probe, add retry loop, capture stderr for scrcpy diagnostics ([8537725](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/8537725b02796cd681f7323b7f14d06c82bb735d))
* replace WebSocket with Tauri events for video streaming ([3f32c8c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/3f32c8c71bac14e02a87220845107c110291193f))
* **scrcpy-server:** improve logging and error handling ([1b49e0b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/1b49e0bda6e63a4ffa5c6069c93cd96768f33c4e))
* **scrcpy:** emit debug logs after frontend listener is set up ([7f91774](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7f91774ec66f76243dd9e24f30fbf10f48ad0684))
* **scrcpy:** remove unsupported --quiet option for older scrcpy versions ([2cd58a1](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2cd58a14dcb501742cc9f3ab676c2d1a306147dc))
* **scrcpy:** use correct port forwarding to abstract socket ([51b1b6f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/51b1b6f05433188588f6178243eb63d6bfa04d66))
* skip postbump hook in standard-version ([956ed9d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/956ed9da0f6b661004063a097ad426f7f011ee6c))
* sync companion release version files ([ccddccd](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ccddccd3e78fc91a0611feb984ebcf2beed93130))
* sync version files to 1.2.8 + add release script ([4617e18](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4617e18a6f8e02b4b223a34ca918e0fcfbe03ba4))
* sync versions to 1.3.0 ([2939bd8](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2939bd827ec6a5b118c527472ac7b3b71202b06c))
* update scrcpy_server and device_controller imports ([01acd9d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/01acd9d8b3d4bde9f8f422f1a401144285fdcd46))
* use .cjs extension for version updater ([d36b420](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/d36b4201e62e7181a324b61f7e8ade0fdd088a23))
* use snake_case for duration_ms in device_swipe ([5f3bb39](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/5f3bb39f1fe0a00f27b6d6035fbe64c151121aae))
* Windows path bug in check-version-sync.ts ([cc9ed5f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/cc9ed5ff748e76e67d387a6ec90520a91ced4270))

## [1.4.0](https://github.com/ManotLuijiu/auto-affiliate-agents/compare/v1.2.4...v1.4.0) (2026-07-30)


### Features

* add mirror mode badge (scrcpy/ADB indicator) ([ded2dc4](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ded2dc438a2227fbed05588139b6c4e018f0c42a))
* add Mirror Relay client support ([0623a09](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0623a092bb482721371b20e2de7c8eda5b56bfbe))
* add Mirror Relay toggle to Settings ([d977516](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/d97751612bd0a04033b7dbacfba8d0475b8b4197))
* add user badge in header showing logged-in email ([7b7ab5a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7b7ab5ae48c6f8642ff1d9fe47eb85134faffe99))
* bundle scrcpy-server and improve path resolution ([2ff9e5d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2ff9e5dcec0f59cf22466d8e896715b3bc4d114f))
* enable real video streaming with screenshot fallback ([297ad28](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/297ad280a9a3361c788ebc04cabebaf1eb4c42f7))
* improve scrcpy UX, secure-screen notice, device icon, friendly names ([2573234](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/25732344b36ab43f0e2f7093886d0af17d62383f))
* **mirror:** add scrcpy-server streaming to #mirror-screen div ([f828603](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f82860333a55525803e61e60ee692eada21b2baf))
* use Google brand PNG logo and update video stream handling ([09f37e6](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/09f37e6e55859ce16390bebb42fa9e575f8d4148))
* use standard-version bumpFiles for multi-file version sync ([9bfeadb](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9bfeadb77b4a845c1400a4d691bf3e773defdbe5))


### Bug Fixes

* add rename_all = snake_case to device_swipe for Tauri arg mapping ([ede1a56](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ede1a56450f254db634641b6c15a540e413164c7))
* apply user changes ([111a884](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/111a88445ec6acbef5c780810e71ef4b092cc5a8))
* implement scrcpy 4.x wire protocol correctly ([515437a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/515437adb66c6508258340b721abff486b07df10))
* improve scrcpy-server process detection ([8ec2b53](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/8ec2b53d6cca1d0a633162a22944eb646cbbcab5))
* **mirror:** fix two critical root causes of live mirror timeout ([455eada](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/455eadac00d76cc900350aace01949a95c295f3b))
* **mirror:** toggle scrcpy now uses scrcpy-server mode ([5fb9811](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/5fb9811cbe156bdc607db07be2a05dbcdd4ca3ff))
* **mirror:** video stream false success state - NAL parsing + first-frame timeout ([dccd3e2](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/dccd3e289fb712708274ea1e0de9777eeb755d13))
* remove nohup, fix device-meta parser for scrcpy 4.x protocol ([1975b9f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/1975b9f592e4ce8303263913731d007d33f22ee8))
* remove TCP probe, add retry loop, capture stderr for scrcpy diagnostics ([8537725](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/8537725b02796cd681f7323b7f14d06c82bb735d))
* replace WebSocket with Tauri events for video streaming ([3f32c8c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/3f32c8c71bac14e02a87220845107c110291193f))
* **scrcpy-server:** improve logging and error handling ([1b49e0b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/1b49e0bda6e63a4ffa5c6069c93cd96768f33c4e))
* **scrcpy:** emit debug logs after frontend listener is set up ([7f91774](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7f91774ec66f76243dd9e24f30fbf10f48ad0684))
* **scrcpy:** remove unsupported --quiet option for older scrcpy versions ([2cd58a1](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2cd58a14dcb501742cc9f3ab676c2d1a306147dc))
* **scrcpy:** use correct port forwarding to abstract socket ([51b1b6f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/51b1b6f05433188588f6178243eb63d6bfa04d66))
* skip postbump hook in standard-version ([956ed9d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/956ed9da0f6b661004063a097ad426f7f011ee6c))
* sync companion release version files ([ccddccd](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ccddccd3e78fc91a0611feb984ebcf2beed93130))
* sync version files to 1.2.8 + add release script ([4617e18](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4617e18a6f8e02b4b223a34ca918e0fcfbe03ba4))
* sync versions to 1.3.0 ([2939bd8](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2939bd827ec6a5b118c527472ac7b3b71202b06c))
* update scrcpy_server and device_controller imports ([01acd9d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/01acd9d8b3d4bde9f8f422f1a401144285fdcd46))
* use snake_case for duration_ms in device_swipe ([5f3bb39](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/5f3bb39f1fe0a00f27b6d6035fbe64c151121aae))
* Windows path bug in check-version-sync.ts ([cc9ed5f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/cc9ed5ff748e76e67d387a6ec90520a91ced4270))


### Documentation

* document working and non-working components ([6ff8f57](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/6ff8f579067dcd41b2b8620dc28db03350564d81))


### Performance

* optimize mirror rendering - use canvas directly, reduce bitrate/fps ([3dd0964](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/3dd0964fab57117f1dc8533fb7f18fb9d9bc8d57))


### Maintenance

* apply user changes ([35dd00e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/35dd00e05badcc3083a6268a2ea93faab76352c2))
* bump to v1.3.1 ([d9bcdc5](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/d9bcdc5b353a88f53ea02a614ba309e892989378))
* bump version to 1.2.26 ([fc89e29](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/fc89e298d69a5fd516e494b509e30dc0cb10c510))
* **release:** 1.2.10 ([9d8424d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9d8424dd35a38b1a99ad477624a0b3e485e31906))
* **release:** 1.2.11 ([bad4f35](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/bad4f35f671eb2ea669102c2e72a30adafe17274))
* **release:** 1.2.12 ([59e3bf4](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/59e3bf4cd373eee409e55c7819bb250f38785f6a))
* **release:** 1.2.13 ([7570971](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7570971c1256f813e80773317632992ed5f81111))
* **release:** 1.2.14 ([21aa1bc](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/21aa1bc1261cde7b70fcde389dc3441e24a32c62))
* **release:** 1.2.15 ([e14461e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/e14461efb86773b04e8ecc4275b17c1f83ebeec3))
* **release:** 1.2.16 ([2344d56](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2344d569f593bc6648730089a2b7c920849e12b2))
* **release:** 1.2.17 ([e1f09d9](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/e1f09d9d9e5f1e5e48709697ef50579a312fa2e7))
* **release:** 1.2.18 ([16138a7](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/16138a7b194918ee158e35f9b3eb992740be5314))
* **release:** 1.2.19 ([e2bbda0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/e2bbda05043c5595aa0fd0f0ba1a327f729279cf))
* **release:** 1.2.20 ([0ac16f4](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0ac16f474f357cd2dbec3b16697cd02c1c6b6ec4))
* **release:** 1.2.21 ([b5ce06f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b5ce06fbdd0869635fddb85023b71692c6636b47))
* **release:** 1.2.22 ([2c015e0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2c015e0d04917a6c84a80ffb2b027257a53e8599))
* **release:** 1.2.23 ([1896c57](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/1896c576ced86bece47ad5499a99391ab18b6a7c))
* **release:** 1.2.24 ([259b104](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/259b104faa3a252947b3b14bf37322bf64ae6895))
* **release:** 1.2.25 ([1676f88](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/1676f885721952628c1f9659f8ba9be24bde9c81))
* **release:** 1.2.28 ([f46e0b2](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f46e0b2b158ab4cc00e083335b5c245c5b2358e1))
* **release:** 1.2.30 ([26a6967](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/26a69672e2d19b06b5194b7a086c6b21a877fe64))
* **release:** 1.2.31 ([4d8d658](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4d8d6582e9c6145a70d721f23a37ab9d085de4ce))
* **release:** 1.2.32 ([8e69506](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/8e69506aa9ed948b5231d837642f7a28ee9b71e5))
* **release:** 1.2.33 ([46813ea](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/46813ea137f2bcd9ee9bfa8118b31c35957e1ed4))
* **release:** 1.2.34 ([f90772e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f90772e70db5d256b1c3100a507b456ce99d2546))
* **release:** 1.2.35 ([831f218](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/831f2183a139dc773901a6d825ba0b68f8b68a74))
* **release:** 1.2.36 ([a95f831](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a95f83144801738f5e5de68771a0aa1ee731fba0))
* **release:** 1.2.37 ([c00d702](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/c00d70283ebebd2d0785b8491065ad629f0b6c03))
* **release:** 1.2.38 ([54b5ece](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/54b5ecea707b24faee4d19696c57f2aa98619992))
* **release:** 1.2.9 ([0f8e379](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0f8e379a2cfa5ad01bdefd7a9b29d3e2cc43d037))
* replace npm version with bun scripts ([231b9a3](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/231b9a36c60d7d5288acc921a6043736c64866ff))
* simplify release scripts with standard-version ([351527b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/351527b93ba8bf4023dcdfc27f962aa079368490))
* sync version files to 1.2.12 ([3817a3a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/3817a3a35064904544cb8f19204720ebec2339b8))

### [1.2.25](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.24...companion/v1.2.25) (2026-07-28)


### Bug Fixes

* **mirror:** fix two critical root causes of live mirror timeout ([455eada](https://github.com/ManotLuijiu/amos-companion/commit/455eadac00d76cc900350aace01949a95c295f3b))

### [1.2.24](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.23...companion/v1.2.24) (2026-07-28)


### Bug Fixes

* **scrcpy:** emit debug logs after frontend listener is set up ([7f91774](https://github.com/ManotLuijiu/amos-companion/commit/7f91774ec66f76243dd9e24f30fbf10f48ad0684))

### [1.2.23](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.22...companion/v1.2.23) (2026-07-28)

### [1.2.22](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.21...companion/v1.2.22) (2026-07-28)

### [1.2.21](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.20...companion/v1.2.21) (2026-07-28)

### [1.2.20](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.19...companion/v1.2.20) (2026-07-28)


### Bug Fixes

* **scrcpy:** use correct port forwarding to abstract socket ([51b1b6f](https://github.com/ManotLuijiu/amos-companion/commit/51b1b6f05433188588f6178243eb63d6bfa04d66))

### [1.2.19](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.18...companion/v1.2.19) (2026-07-28)

### [1.2.18](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.17...companion/v1.2.18) (2026-07-28)

### [1.2.17](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.16...companion/v1.2.17) (2026-07-28)

### [1.2.16](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.15...companion/v1.2.16) (2026-07-28)

### [1.2.15](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.14...companion/v1.2.15) (2026-07-28)


### Bug Fixes

* **scrcpy-server:** improve logging and error handling ([1b49e0b](https://github.com/ManotLuijiu/amos-companion/commit/1b49e0bda6e63a4ffa5c6069c93cd96768f33c4e))

### [1.2.14](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.13...companion/v1.2.14) (2026-07-28)


### Bug Fixes

* **mirror:** toggle scrcpy now uses scrcpy-server mode ([5fb9811](https://github.com/ManotLuijiu/amos-companion/commit/5fb9811cbe156bdc607db07be2a05dbcdd4ca3ff))

### [1.2.13](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.12...companion/v1.2.13) (2026-07-28)


### Features

* **mirror:** add scrcpy-server streaming to #mirror-screen div ([f828603](https://github.com/ManotLuijiu/amos-companion/commit/f82860333a55525803e61e60ee692eada21b2baf))


### Bug Fixes

* update scrcpy_server and device_controller imports ([01acd9d](https://github.com/ManotLuijiu/amos-companion/commit/01acd9d8b3d4bde9f8f422f1a401144285fdcd46))


### Maintenance

* sync version files to 1.2.12 ([3817a3a](https://github.com/ManotLuijiu/amos-companion/commit/3817a3a35064904544cb8f19204720ebec2339b8))


### Documentation

* document working and non-working components ([6ff8f57](https://github.com/ManotLuijiu/amos-companion/commit/6ff8f579067dcd41b2b8620dc28db03350564d81))

### [1.2.12](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.11...companion/v1.2.12) (2026-07-28)


### Bug Fixes

* **scrcpy:** remove unsupported --quiet option for older scrcpy versions ([2cd58a1](https://github.com/ManotLuijiu/amos-companion/commit/2cd58a14dcb501742cc9f3ab676c2d1a306147dc))

### [1.2.11](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.10...companion/v1.2.11) (2026-07-28)


### Bug Fixes

* **mirror:** video stream false success state - NAL parsing + first-frame timeout ([dccd3e2](https://github.com/ManotLuijiu/amos-companion/commit/dccd3e289fb712708274ea1e0de9777eeb755d13))

### [1.2.10](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.9...companion/v1.2.10) (2026-07-28)


### Bug Fixes

* replace WebSocket with Tauri events for video streaming ([3f32c8c](https://github.com/ManotLuijiu/amos-companion/commit/3f32c8c71bac14e02a87220845107c110291193f))

### [1.2.9](https://github.com/ManotLuijiu/amos-companion/compare/companion/v1.2.8...companion/v1.2.9) (2026-07-28)


### Features

* enable real video streaming with screenshot fallback ([297ad28](https://github.com/ManotLuijiu/amos-companion/commit/297ad280a9a3361c788ebc04cabebaf1eb4c42f7))


### Bug Fixes

* sync version files to 1.2.8 + add release script ([4617e18](https://github.com/ManotLuijiu/amos-companion/commit/4617e18a6f8e02b4b223a34ca918e0fcfbe03ba4))

### [1.2.4](https://github.com/ManotLuijiu/auto-affiliate-agents/compare/v1.0.0...v1.2.4) (2026-07-27)


### Features

* Add email/password login flow for AMOS Companion ([76c3e8c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/76c3e8ceeec052fa5d26efa0ce8d23164a28d400))
* Add Google OAuth login option ([240bd00](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/240bd00aea3c60543a0fa6c25fb55946ddda6178))
* Add manual fallback for OAuth flow ([54a6d1a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/54a6d1a05e7f620d6e110a92499f8e4ae0643059))
* Add OAuth sign-in flow with local callback server ([3396c60](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/3396c609c047149cf72f6a191a0ce38e2f133519))
* Add Playwright E2E testing ([22282ca](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/22282cacbc46bc668931d717bfaa17d54129ec46))
* Add single-source version management ([0b6c8f7](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0b6c8f7342df07cdc60f44b7998914593a3bed2e))
* Add version sync and AMOS logo ([39b764d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/39b764d20c5ca17ff996bd9cc74ed72169214217))
* Add video_stream module with WebSocket streaming support ([fd14c52](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/fd14c5226891d648972c0fa00700d38f3a3acb29))
* Complete UI redesign - 3-panel layout ([d88849d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/d88849d2c84657c8fd51a19d6be52a3a0f004774))
* Implement mirror with screenshot polling ([2f43c40](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2f43c40b2571dfc19a1d8e5f40a5f7b69e5fa77d))
* Implement true WebSocket video streaming ([2c3e24c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2c3e24cd4d31eb48e689869b6b15310375a121e3))
* Inject app version into frontend at build time ([4fe9e9b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4fe9e9b7457308875b5601a98c35e752f3b312dd))
* Switch to standard-version for version management ([4975e91](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4975e917f6bb01c22c7d1df437209ec0006301b3))
* WebSocket-ready scrcpy-server with streaming support ([c53eb4c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/c53eb4c7a144f915703c6e9908b56c9197bb30d5))


### Bug Fixes

* Add AWS credentials to workflow environment ([f919afd](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f919afd57c8c6bfb32ba81a17af00feee2afdc44))
* Add cleanup job to keep only last 3 builds per platform ([089a6f2](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/089a6f25dfd81fd91ed06b4e4593e74c8f355322))
* Add detailed error logging for screenshot capture ([b94c517](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b94c517edc0b0b4955cc1ba5aa75ea5af2db71a5))
* Add detailed WebSocket debug logging ([a22041e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a22041ef419c7b60b96b976035786c6a972ea4f5))
* Add detailed WebSocket debug logging ([6c8c637](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/6c8c63781ce4ea37bdada0c5eef0008114a81793))
* add missing Duration import for macOS build ([7d2c338](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7d2c338a9a75ee9f8fabf441d03b8ef4cb91a5d9))
* add missing Duration import for macOS build ([c296d05](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/c296d053b0ca13db72d22c35bc740bc34009aa16))
* add missing update-version.ts script for GitHub Actions ([1b0e7a5](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/1b0e7a53d41d23b735fc4f0a8578c521faf171c7))
* Add RELEASE_TOKEN to all GitHub release steps ([a18a216](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a18a2167aba58c438b3cee6e20e6c370a6c1e178))
* Add screenshot error logging ([f0a4d88](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f0a4d88f5d25cb118efe09a7bc5ac84e82bff013))
* add swipe gestures and fix agent status PID consistency ([8f0fb10](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/8f0fb100fcd32f593b886d6355a7e7341825b3b3))
* Add write permissions for GitHub releases ([0788ea0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0788ea079d59b6811d4d5314a15fbc51748dda37))
* built-in mirror touch coordinate mapping ([89d909c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/89d909c3a794d246599a8df566e01d89269b4b2d))
* checkout main branch in update-version workflow to avoid detached HEAD ([0695cf0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0695cf0e22f9922629e44c3865f97d3ee3f3a79e))
* Cleanup by version number, not timestamp ([ae4c043](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ae4c0438cf067a37c102b8c5123583fcbf0a57ce))
* Cleanup keeps newest 3 builds per platform ([cf72c17](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/cf72c179f767a4eba0d3d81f7e6a5efa4ddf6909))
* complete companion swipe and release workflow ([f198b11](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f198b11b6801951a34f03d123c5552014e6c8d6b))
* convert .versionrc.js and cargo-updater.js to ES modules ([ba15e69](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ba15e695ef04e6256be87814adde91632fa621a8))
* Correct AWS CLI v2 macOS installer command ([a12272b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a12272bdc586a321e3a97fe3ef26f528d8c74da0))
* Correct Google OAuth URL conversion from API to frontend ([b2cbbf0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b2cbbf0c2a7bf87da32c79713d08d9b48b4a68aa))
* Device click now starts mirroring properly ([94f8b8d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/94f8b8d49d03c80bbd8b5b65429698bba903ffd4))
* Disable WebSocket streaming, use screenshot polling only ([92c082d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/92c082d8215be938b682e890b4ba6f06172f9dc7))
* Get API URL from outside the form in handleLogin ([81209c8](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/81209c8a6a943970ba3351f56f1d382ab206d106))
* Handle amos-api.moo-vpn.online URL in OAuth flow ([b2f3f6b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b2f3f6bcc24e7fb1eb46c2e1ce0e18f772471e29))
* install create-dmg for macOS DMG bundling ([20d7b92](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/20d7b92462d0f793c4bbe6a00bbc38f6acc01d6a))
* Listen for login-success event and show main content after OAuth ([32c25df](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/32c25df5db81144a161b7f92fe74c6187eae9ace))
* mirror touch control and agent status tracking ([7f12d68](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7f12d68f3dca9cd26c0d849c22227430c0b8bdf9))
* pointer capture for swipe gestures + agent_running truth source ([963d0f6](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/963d0f66e3598abed64e2a615c08b907aac9c9eb))
* Remove inline style.display block + stop polling on errors ([23025b0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/23025b0403561453dedc01b15363bd5b6657042c))
* resolve 4 bugs - mirror blank panel, scrcpy UX, stale version, agent status ([2951813](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2951813815c75c8c1f2c583ad3982da031e9f8fa))
* Restore scrcpyEnabled variable name ([7bc0268](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7bc0268fe395de23a29d11404a751b0133d00661))
* Send proper WebSocket frames for video streaming ([70adcbe](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/70adcbe0ea7f74407ad86944b8a901b7bb416552))
* Set correct default API URL in config_store ([04b15ff](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/04b15ff3fba855b872d25b81d2143ffc2955715e))
* simplify agent_running truth source ([cca5a99](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/cca5a990bd05264c3b7f72419591291a340bfe34))
* Simplify AWS CLI v2 macOS install to match official docs ([a35d13c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a35d13c79fd80c43904c1e6e472794f5cca48338))
* Stop screenshot polling after 3 errors ([30ae079](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/30ae07998d67e078b833dafb5032b659a94b4c45))
* Update Linux AWS CLI install with --update flag ([9f51635](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9f51635949404b9f95b8dd237aac78c1a0cc590c))
* Update version to 1.0.11 in Cargo.toml and package.json ([547687a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/547687a85ab1c91ab5e45604d5ca4980f88722a5))
* Use AWS CLI v2 installer instead of pip for macOS and Windows ([abdf70f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/abdf70f024d7b97096e7e5a5c419871033a09113))
* Use correct API URL amos-api.moo-vpn.online ([0d9d0d4](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0d9d0d4df3e2b916bcde1a227a0fc9d3146449df))
* Use correct better-auth OAuth flow with callbackUrl ([9ca45fb](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9ca45fb01157881b03cbfeadb54d11525bd54964))
* Use msiexec for Windows AWS CLI v2 installation ([77cf2ba](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/77cf2ba660b8d10255f7ee2548467975947c62a7))
* Use official AWS CLI v2 Linux installer command ([abcee54](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/abcee549b56a2d766f343a39ebe87344f4e3d003))
* Use port 0 for dynamic port binding ([76f404e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/76f404e8b168be7801fc4116bdc05473fe219099))

### [1.2.3](https://github.com/ManotLuijiu/auto-affiliate-agents/compare/v1.0.0...v1.2.3) (2026-07-27)


### Features

* Add email/password login flow for AMOS Companion ([76c3e8c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/76c3e8ceeec052fa5d26efa0ce8d23164a28d400))
* Add Google OAuth login option ([240bd00](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/240bd00aea3c60543a0fa6c25fb55946ddda6178))
* Add manual fallback for OAuth flow ([54a6d1a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/54a6d1a05e7f620d6e110a92499f8e4ae0643059))
* Add OAuth sign-in flow with local callback server ([3396c60](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/3396c609c047149cf72f6a191a0ce38e2f133519))
* Add Playwright E2E testing ([22282ca](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/22282cacbc46bc668931d717bfaa17d54129ec46))
* Add single-source version management ([0b6c8f7](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0b6c8f7342df07cdc60f44b7998914593a3bed2e))
* Add version sync and AMOS logo ([39b764d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/39b764d20c5ca17ff996bd9cc74ed72169214217))
* Add video_stream module with WebSocket streaming support ([fd14c52](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/fd14c5226891d648972c0fa00700d38f3a3acb29))
* Complete UI redesign - 3-panel layout ([d88849d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/d88849d2c84657c8fd51a19d6be52a3a0f004774))
* Implement mirror with screenshot polling ([2f43c40](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2f43c40b2571dfc19a1d8e5f40a5f7b69e5fa77d))
* Implement true WebSocket video streaming ([2c3e24c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2c3e24cd4d31eb48e689869b6b15310375a121e3))
* Inject app version into frontend at build time ([4fe9e9b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4fe9e9b7457308875b5601a98c35e752f3b312dd))
* Switch to standard-version for version management ([4975e91](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4975e917f6bb01c22c7d1df437209ec0006301b3))
* WebSocket-ready scrcpy-server with streaming support ([c53eb4c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/c53eb4c7a144f915703c6e9908b56c9197bb30d5))


### Bug Fixes

* Add AWS credentials to workflow environment ([f919afd](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f919afd57c8c6bfb32ba81a17af00feee2afdc44))
* Add cleanup job to keep only last 3 builds per platform ([089a6f2](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/089a6f25dfd81fd91ed06b4e4593e74c8f355322))
* Add detailed error logging for screenshot capture ([b94c517](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b94c517edc0b0b4955cc1ba5aa75ea5af2db71a5))
* Add detailed WebSocket debug logging ([a22041e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a22041ef419c7b60b96b976035786c6a972ea4f5))
* Add detailed WebSocket debug logging ([6c8c637](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/6c8c63781ce4ea37bdada0c5eef0008114a81793))
* add missing Duration import for macOS build ([c296d05](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/c296d053b0ca13db72d22c35bc740bc34009aa16))
* add missing update-version.ts script for GitHub Actions ([1b0e7a5](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/1b0e7a53d41d23b735fc4f0a8578c521faf171c7))
* Add RELEASE_TOKEN to all GitHub release steps ([a18a216](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a18a2167aba58c438b3cee6e20e6c370a6c1e178))
* Add screenshot error logging ([f0a4d88](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f0a4d88f5d25cb118efe09a7bc5ac84e82bff013))
* add swipe gestures and fix agent status PID consistency ([8f0fb10](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/8f0fb100fcd32f593b886d6355a7e7341825b3b3))
* Add write permissions for GitHub releases ([0788ea0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0788ea079d59b6811d4d5314a15fbc51748dda37))
* built-in mirror touch coordinate mapping ([89d909c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/89d909c3a794d246599a8df566e01d89269b4b2d))
* Cleanup by version number, not timestamp ([ae4c043](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ae4c0438cf067a37c102b8c5123583fcbf0a57ce))
* Cleanup keeps newest 3 builds per platform ([cf72c17](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/cf72c179f767a4eba0d3d81f7e6a5efa4ddf6909))
* convert .versionrc.js and cargo-updater.js to ES modules ([ba15e69](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ba15e695ef04e6256be87814adde91632fa621a8))
* Correct AWS CLI v2 macOS installer command ([a12272b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a12272bdc586a321e3a97fe3ef26f528d8c74da0))
* Correct Google OAuth URL conversion from API to frontend ([b2cbbf0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b2cbbf0c2a7bf87da32c79713d08d9b48b4a68aa))
* Device click now starts mirroring properly ([94f8b8d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/94f8b8d49d03c80bbd8b5b65429698bba903ffd4))
* Disable WebSocket streaming, use screenshot polling only ([92c082d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/92c082d8215be938b682e890b4ba6f06172f9dc7))
* Get API URL from outside the form in handleLogin ([81209c8](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/81209c8a6a943970ba3351f56f1d382ab206d106))
* Handle amos-api.moo-vpn.online URL in OAuth flow ([b2f3f6b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b2f3f6bcc24e7fb1eb46c2e1ce0e18f772471e29))
* Listen for login-success event and show main content after OAuth ([32c25df](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/32c25df5db81144a161b7f92fe74c6187eae9ace))
* mirror touch control and agent status tracking ([7f12d68](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7f12d68f3dca9cd26c0d849c22227430c0b8bdf9))
* Remove inline style.display block + stop polling on errors ([23025b0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/23025b0403561453dedc01b15363bd5b6657042c))
* resolve 4 bugs - mirror blank panel, scrcpy UX, stale version, agent status ([2951813](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2951813815c75c8c1f2c583ad3982da031e9f8fa))
* Restore scrcpyEnabled variable name ([7bc0268](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7bc0268fe395de23a29d11404a751b0133d00661))
* Send proper WebSocket frames for video streaming ([70adcbe](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/70adcbe0ea7f74407ad86944b8a901b7bb416552))
* Set correct default API URL in config_store ([04b15ff](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/04b15ff3fba855b872d25b81d2143ffc2955715e))
* Simplify AWS CLI v2 macOS install to match official docs ([a35d13c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a35d13c79fd80c43904c1e6e472794f5cca48338))
* Stop screenshot polling after 3 errors ([30ae079](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/30ae07998d67e078b833dafb5032b659a94b4c45))
* Update Linux AWS CLI install with --update flag ([9f51635](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9f51635949404b9f95b8dd237aac78c1a0cc590c))
* Update version to 1.0.11 in Cargo.toml and package.json ([547687a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/547687a85ab1c91ab5e45604d5ca4980f88722a5))
* Use AWS CLI v2 installer instead of pip for macOS and Windows ([abdf70f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/abdf70f024d7b97096e7e5a5c419871033a09113))
* Use correct API URL amos-api.moo-vpn.online ([0d9d0d4](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0d9d0d4df3e2b916bcde1a227a0fc9d3146449df))
* Use correct better-auth OAuth flow with callbackUrl ([9ca45fb](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9ca45fb01157881b03cbfeadb54d11525bd54964))
* Use msiexec for Windows AWS CLI v2 installation ([77cf2ba](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/77cf2ba660b8d10255f7ee2548467975947c62a7))
* Use official AWS CLI v2 Linux installer command ([abcee54](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/abcee549b56a2d766f343a39ebe87344f4e3d003))
* Use port 0 for dynamic port binding ([76f404e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/76f404e8b168be7801fc4116bdc05473fe219099))

### [1.2.2](https://github.com/ManotLuijiu/auto-affiliate-agents/compare/v1.0.0...v1.2.2) (2026-07-27)


### Features

* Add email/password login flow for AMOS Companion ([76c3e8c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/76c3e8ceeec052fa5d26efa0ce8d23164a28d400))
* Add Google OAuth login option ([240bd00](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/240bd00aea3c60543a0fa6c25fb55946ddda6178))
* Add manual fallback for OAuth flow ([54a6d1a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/54a6d1a05e7f620d6e110a92499f8e4ae0643059))
* Add OAuth sign-in flow with local callback server ([3396c60](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/3396c609c047149cf72f6a191a0ce38e2f133519))
* Add Playwright E2E testing ([22282ca](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/22282cacbc46bc668931d717bfaa17d54129ec46))
* Add single-source version management ([0b6c8f7](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0b6c8f7342df07cdc60f44b7998914593a3bed2e))
* Add version sync and AMOS logo ([39b764d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/39b764d20c5ca17ff996bd9cc74ed72169214217))
* Add video_stream module with WebSocket streaming support ([fd14c52](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/fd14c5226891d648972c0fa00700d38f3a3acb29))
* Complete UI redesign - 3-panel layout ([d88849d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/d88849d2c84657c8fd51a19d6be52a3a0f004774))
* Implement mirror with screenshot polling ([2f43c40](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2f43c40b2571dfc19a1d8e5f40a5f7b69e5fa77d))
* Implement true WebSocket video streaming ([2c3e24c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2c3e24cd4d31eb48e689869b6b15310375a121e3))
* Inject app version into frontend at build time ([4fe9e9b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4fe9e9b7457308875b5601a98c35e752f3b312dd))
* Switch to standard-version for version management ([4975e91](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4975e917f6bb01c22c7d1df437209ec0006301b3))
* WebSocket-ready scrcpy-server with streaming support ([c53eb4c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/c53eb4c7a144f915703c6e9908b56c9197bb30d5))


### Bug Fixes

* Add AWS credentials to workflow environment ([f919afd](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f919afd57c8c6bfb32ba81a17af00feee2afdc44))
* Add cleanup job to keep only last 3 builds per platform ([089a6f2](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/089a6f25dfd81fd91ed06b4e4593e74c8f355322))
* Add detailed error logging for screenshot capture ([b94c517](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b94c517edc0b0b4955cc1ba5aa75ea5af2db71a5))
* Add detailed WebSocket debug logging ([a22041e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a22041ef419c7b60b96b976035786c6a972ea4f5))
* Add detailed WebSocket debug logging ([6c8c637](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/6c8c63781ce4ea37bdada0c5eef0008114a81793))
* add missing Duration import for macOS build ([c296d05](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/c296d053b0ca13db72d22c35bc740bc34009aa16))
* add missing update-version.ts script for GitHub Actions ([1b0e7a5](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/1b0e7a53d41d23b735fc4f0a8578c521faf171c7))
* Add RELEASE_TOKEN to all GitHub release steps ([a18a216](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a18a2167aba58c438b3cee6e20e6c370a6c1e178))
* Add screenshot error logging ([f0a4d88](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f0a4d88f5d25cb118efe09a7bc5ac84e82bff013))
* Add write permissions for GitHub releases ([0788ea0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0788ea079d59b6811d4d5314a15fbc51748dda37))
* built-in mirror touch coordinate mapping ([89d909c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/89d909c3a794d246599a8df566e01d89269b4b2d))
* Cleanup by version number, not timestamp ([ae4c043](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ae4c0438cf067a37c102b8c5123583fcbf0a57ce))
* Cleanup keeps newest 3 builds per platform ([cf72c17](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/cf72c179f767a4eba0d3d81f7e6a5efa4ddf6909))
* convert .versionrc.js and cargo-updater.js to ES modules ([ba15e69](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ba15e695ef04e6256be87814adde91632fa621a8))
* Correct AWS CLI v2 macOS installer command ([a12272b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a12272bdc586a321e3a97fe3ef26f528d8c74da0))
* Correct Google OAuth URL conversion from API to frontend ([b2cbbf0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b2cbbf0c2a7bf87da32c79713d08d9b48b4a68aa))
* Device click now starts mirroring properly ([94f8b8d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/94f8b8d49d03c80bbd8b5b65429698bba903ffd4))
* Disable WebSocket streaming, use screenshot polling only ([92c082d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/92c082d8215be938b682e890b4ba6f06172f9dc7))
* Get API URL from outside the form in handleLogin ([81209c8](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/81209c8a6a943970ba3351f56f1d382ab206d106))
* Handle amos-api.moo-vpn.online URL in OAuth flow ([b2f3f6b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b2f3f6bcc24e7fb1eb46c2e1ce0e18f772471e29))
* Listen for login-success event and show main content after OAuth ([32c25df](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/32c25df5db81144a161b7f92fe74c6187eae9ace))
* mirror touch control and agent status tracking ([7f12d68](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7f12d68f3dca9cd26c0d849c22227430c0b8bdf9))
* Remove inline style.display block + stop polling on errors ([23025b0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/23025b0403561453dedc01b15363bd5b6657042c))
* resolve 4 bugs - mirror blank panel, scrcpy UX, stale version, agent status ([2951813](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2951813815c75c8c1f2c583ad3982da031e9f8fa))
* Restore scrcpyEnabled variable name ([7bc0268](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7bc0268fe395de23a29d11404a751b0133d00661))
* Send proper WebSocket frames for video streaming ([70adcbe](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/70adcbe0ea7f74407ad86944b8a901b7bb416552))
* Set correct default API URL in config_store ([04b15ff](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/04b15ff3fba855b872d25b81d2143ffc2955715e))
* Simplify AWS CLI v2 macOS install to match official docs ([a35d13c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a35d13c79fd80c43904c1e6e472794f5cca48338))
* Stop screenshot polling after 3 errors ([30ae079](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/30ae07998d67e078b833dafb5032b659a94b4c45))
* Update Linux AWS CLI install with --update flag ([9f51635](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9f51635949404b9f95b8dd237aac78c1a0cc590c))
* Update version to 1.0.11 in Cargo.toml and package.json ([547687a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/547687a85ab1c91ab5e45604d5ca4980f88722a5))
* Use AWS CLI v2 installer instead of pip for macOS and Windows ([abdf70f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/abdf70f024d7b97096e7e5a5c419871033a09113))
* Use correct API URL amos-api.moo-vpn.online ([0d9d0d4](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0d9d0d4df3e2b916bcde1a227a0fc9d3146449df))
* Use correct better-auth OAuth flow with callbackUrl ([9ca45fb](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9ca45fb01157881b03cbfeadb54d11525bd54964))
* Use msiexec for Windows AWS CLI v2 installation ([77cf2ba](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/77cf2ba660b8d10255f7ee2548467975947c62a7))
* Use official AWS CLI v2 Linux installer command ([abcee54](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/abcee549b56a2d766f343a39ebe87344f4e3d003))
* Use port 0 for dynamic port binding ([76f404e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/76f404e8b168be7801fc4116bdc05473fe219099))

## [1.2.0](https://github.com/ManotLuijiu/auto-affiliate-agents/compare/v1.0.0...v1.2.0) (2026-07-27)


### Features

* Add email/password login flow for AMOS Companion ([76c3e8c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/76c3e8ceeec052fa5d26efa0ce8d23164a28d400))
* Add Google OAuth login option ([240bd00](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/240bd00aea3c60543a0fa6c25fb55946ddda6178))
* Add manual fallback for OAuth flow ([54a6d1a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/54a6d1a05e7f620d6e110a92499f8e4ae0643059))
* Add OAuth sign-in flow with local callback server ([3396c60](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/3396c609c047149cf72f6a191a0ce38e2f133519))
* Add Playwright E2E testing ([22282ca](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/22282cacbc46bc668931d717bfaa17d54129ec46))
* Add single-source version management ([0b6c8f7](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0b6c8f7342df07cdc60f44b7998914593a3bed2e))
* Add version sync and AMOS logo ([39b764d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/39b764d20c5ca17ff996bd9cc74ed72169214217))
* Add video_stream module with WebSocket streaming support ([fd14c52](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/fd14c5226891d648972c0fa00700d38f3a3acb29))
* Complete UI redesign - 3-panel layout ([d88849d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/d88849d2c84657c8fd51a19d6be52a3a0f004774))
* Implement mirror with screenshot polling ([2f43c40](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2f43c40b2571dfc19a1d8e5f40a5f7b69e5fa77d))
* Implement true WebSocket video streaming ([2c3e24c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2c3e24cd4d31eb48e689869b6b15310375a121e3))
* Inject app version into frontend at build time ([4fe9e9b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4fe9e9b7457308875b5601a98c35e752f3b312dd))
* Switch to standard-version for version management ([4975e91](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4975e917f6bb01c22c7d1df437209ec0006301b3))
* WebSocket-ready scrcpy-server with streaming support ([c53eb4c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/c53eb4c7a144f915703c6e9908b56c9197bb30d5))


### Bug Fixes

* Add AWS credentials to workflow environment ([f919afd](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f919afd57c8c6bfb32ba81a17af00feee2afdc44))
* Add cleanup job to keep only last 3 builds per platform ([089a6f2](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/089a6f25dfd81fd91ed06b4e4593e74c8f355322))
* Add detailed error logging for screenshot capture ([b94c517](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b94c517edc0b0b4955cc1ba5aa75ea5af2db71a5))
* Add detailed WebSocket debug logging ([a22041e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a22041ef419c7b60b96b976035786c6a972ea4f5))
* Add detailed WebSocket debug logging ([6c8c637](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/6c8c63781ce4ea37bdada0c5eef0008114a81793))
* Add RELEASE_TOKEN to all GitHub release steps ([a18a216](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a18a2167aba58c438b3cee6e20e6c370a6c1e178))
* Add screenshot error logging ([f0a4d88](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f0a4d88f5d25cb118efe09a7bc5ac84e82bff013))
* Add write permissions for GitHub releases ([0788ea0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0788ea079d59b6811d4d5314a15fbc51748dda37))
* Cleanup by version number, not timestamp ([ae4c043](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ae4c0438cf067a37c102b8c5123583fcbf0a57ce))
* Cleanup keeps newest 3 builds per platform ([cf72c17](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/cf72c179f767a4eba0d3d81f7e6a5efa4ddf6909))
* convert .versionrc.js and cargo-updater.js to ES modules ([ba15e69](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ba15e695ef04e6256be87814adde91632fa621a8))
* Correct AWS CLI v2 macOS installer command ([a12272b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a12272bdc586a321e3a97fe3ef26f528d8c74da0))
* Correct Google OAuth URL conversion from API to frontend ([b2cbbf0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b2cbbf0c2a7bf87da32c79713d08d9b48b4a68aa))
* Device click now starts mirroring properly ([94f8b8d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/94f8b8d49d03c80bbd8b5b65429698bba903ffd4))
* Disable WebSocket streaming, use screenshot polling only ([92c082d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/92c082d8215be938b682e890b4ba6f06172f9dc7))
* Get API URL from outside the form in handleLogin ([81209c8](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/81209c8a6a943970ba3351f56f1d382ab206d106))
* Handle amos-api.moo-vpn.online URL in OAuth flow ([b2f3f6b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b2f3f6bcc24e7fb1eb46c2e1ce0e18f772471e29))
* Listen for login-success event and show main content after OAuth ([32c25df](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/32c25df5db81144a161b7f92fe74c6187eae9ace))
* mirror touch control and agent status tracking ([7f12d68](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7f12d68f3dca9cd26c0d849c22227430c0b8bdf9))
* Remove inline style.display block + stop polling on errors ([23025b0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/23025b0403561453dedc01b15363bd5b6657042c))
* resolve 4 bugs - mirror blank panel, scrcpy UX, stale version, agent status ([2951813](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2951813815c75c8c1f2c583ad3982da031e9f8fa))
* Restore scrcpyEnabled variable name ([7bc0268](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7bc0268fe395de23a29d11404a751b0133d00661))
* Send proper WebSocket frames for video streaming ([70adcbe](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/70adcbe0ea7f74407ad86944b8a901b7bb416552))
* Set correct default API URL in config_store ([04b15ff](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/04b15ff3fba855b872d25b81d2143ffc2955715e))
* Simplify AWS CLI v2 macOS install to match official docs ([a35d13c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a35d13c79fd80c43904c1e6e472794f5cca48338))
* Stop screenshot polling after 3 errors ([30ae079](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/30ae07998d67e078b833dafb5032b659a94b4c45))
* Update Linux AWS CLI install with --update flag ([9f51635](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9f51635949404b9f95b8dd237aac78c1a0cc590c))
* Update version to 1.0.11 in Cargo.toml and package.json ([547687a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/547687a85ab1c91ab5e45604d5ca4980f88722a5))
* Use AWS CLI v2 installer instead of pip for macOS and Windows ([abdf70f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/abdf70f024d7b97096e7e5a5c419871033a09113))
* Use correct API URL amos-api.moo-vpn.online ([0d9d0d4](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0d9d0d4df3e2b916bcde1a227a0fc9d3146449df))
* Use correct better-auth OAuth flow with callbackUrl ([9ca45fb](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9ca45fb01157881b03cbfeadb54d11525bd54964))
* Use msiexec for Windows AWS CLI v2 installation ([77cf2ba](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/77cf2ba660b8d10255f7ee2548467975947c62a7))
* Use official AWS CLI v2 Linux installer command ([abcee54](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/abcee549b56a2d766f343a39ebe87344f4e3d003))
* Use port 0 for dynamic port binding ([76f404e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/76f404e8b168be7801fc4116bdc05473fe219099))

## [1.1.0](https://github.com/ManotLuijiu/auto-affiliate-agents/compare/v1.0.0...v1.1.0) (2026-07-27)


### Features

* Add email/password login flow for AMOS Companion ([76c3e8c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/76c3e8ceeec052fa5d26efa0ce8d23164a28d400))
* Add Google OAuth login option ([240bd00](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/240bd00aea3c60543a0fa6c25fb55946ddda6178))
* Add manual fallback for OAuth flow ([54a6d1a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/54a6d1a05e7f620d6e110a92499f8e4ae0643059))
* Add OAuth sign-in flow with local callback server ([3396c60](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/3396c609c047149cf72f6a191a0ce38e2f133519))
* Add Playwright E2E testing ([22282ca](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/22282cacbc46bc668931d717bfaa17d54129ec46))
* Add single-source version management ([0b6c8f7](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0b6c8f7342df07cdc60f44b7998914593a3bed2e))
* Add version sync and AMOS logo ([39b764d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/39b764d20c5ca17ff996bd9cc74ed72169214217))
* Add video_stream module with WebSocket streaming support ([fd14c52](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/fd14c5226891d648972c0fa00700d38f3a3acb29))
* Complete UI redesign - 3-panel layout ([d88849d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/d88849d2c84657c8fd51a19d6be52a3a0f004774))
* Implement mirror with screenshot polling ([2f43c40](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2f43c40b2571dfc19a1d8e5f40a5f7b69e5fa77d))
* Implement true WebSocket video streaming ([2c3e24c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2c3e24cd4d31eb48e689869b6b15310375a121e3))
* Inject app version into frontend at build time ([4fe9e9b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4fe9e9b7457308875b5601a98c35e752f3b312dd))
* Switch to standard-version for version management ([4975e91](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/4975e917f6bb01c22c7d1df437209ec0006301b3))
* WebSocket-ready scrcpy-server with streaming support ([c53eb4c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/c53eb4c7a144f915703c6e9908b56c9197bb30d5))


### Bug Fixes

* Add AWS credentials to workflow environment ([f919afd](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f919afd57c8c6bfb32ba81a17af00feee2afdc44))
* Add cleanup job to keep only last 3 builds per platform ([089a6f2](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/089a6f25dfd81fd91ed06b4e4593e74c8f355322))
* Add detailed error logging for screenshot capture ([b94c517](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b94c517edc0b0b4955cc1ba5aa75ea5af2db71a5))
* Add detailed WebSocket debug logging ([a22041e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a22041ef419c7b60b96b976035786c6a972ea4f5))
* Add detailed WebSocket debug logging ([6c8c637](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/6c8c63781ce4ea37bdada0c5eef0008114a81793))
* Add RELEASE_TOKEN to all GitHub release steps ([a18a216](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a18a2167aba58c438b3cee6e20e6c370a6c1e178))
* Add screenshot error logging ([f0a4d88](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/f0a4d88f5d25cb118efe09a7bc5ac84e82bff013))
* Add write permissions for GitHub releases ([0788ea0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0788ea079d59b6811d4d5314a15fbc51748dda37))
* Cleanup by version number, not timestamp ([ae4c043](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/ae4c0438cf067a37c102b8c5123583fcbf0a57ce))
* Cleanup keeps newest 3 builds per platform ([cf72c17](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/cf72c179f767a4eba0d3d81f7e6a5efa4ddf6909))
* Correct AWS CLI v2 macOS installer command ([a12272b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a12272bdc586a321e3a97fe3ef26f528d8c74da0))
* Correct Google OAuth URL conversion from API to frontend ([b2cbbf0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b2cbbf0c2a7bf87da32c79713d08d9b48b4a68aa))
* Device click now starts mirroring properly ([94f8b8d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/94f8b8d49d03c80bbd8b5b65429698bba903ffd4))
* Disable WebSocket streaming, use screenshot polling only ([92c082d](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/92c082d8215be938b682e890b4ba6f06172f9dc7))
* Get API URL from outside the form in handleLogin ([81209c8](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/81209c8a6a943970ba3351f56f1d382ab206d106))
* Handle amos-api.moo-vpn.online URL in OAuth flow ([b2f3f6b](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/b2f3f6bcc24e7fb1eb46c2e1ce0e18f772471e29))
* Listen for login-success event and show main content after OAuth ([32c25df](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/32c25df5db81144a161b7f92fe74c6187eae9ace))
* Remove inline style.display block + stop polling on errors ([23025b0](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/23025b0403561453dedc01b15363bd5b6657042c))
* resolve 4 bugs - mirror blank panel, scrcpy UX, stale version, agent status ([2951813](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/2951813815c75c8c1f2c583ad3982da031e9f8fa))
* Restore scrcpyEnabled variable name ([7bc0268](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/7bc0268fe395de23a29d11404a751b0133d00661))
* Send proper WebSocket frames for video streaming ([70adcbe](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/70adcbe0ea7f74407ad86944b8a901b7bb416552))
* Set correct default API URL in config_store ([04b15ff](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/04b15ff3fba855b872d25b81d2143ffc2955715e))
* Simplify AWS CLI v2 macOS install to match official docs ([a35d13c](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/a35d13c79fd80c43904c1e6e472794f5cca48338))
* Stop screenshot polling after 3 errors ([30ae079](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/30ae07998d67e078b833dafb5032b659a94b4c45))
* Update Linux AWS CLI install with --update flag ([9f51635](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9f51635949404b9f95b8dd237aac78c1a0cc590c))
* Update version to 1.0.11 in Cargo.toml and package.json ([547687a](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/547687a85ab1c91ab5e45604d5ca4980f88722a5))
* Use AWS CLI v2 installer instead of pip for macOS and Windows ([abdf70f](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/abdf70f024d7b97096e7e5a5c419871033a09113))
* Use correct API URL amos-api.moo-vpn.online ([0d9d0d4](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/0d9d0d4df3e2b916bcde1a227a0fc9d3146449df))
* Use correct better-auth OAuth flow with callbackUrl ([9ca45fb](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/9ca45fb01157881b03cbfeadb54d11525bd54964))
* Use msiexec for Windows AWS CLI v2 installation ([77cf2ba](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/77cf2ba660b8d10255f7ee2548467975947c62a7))
* Use official AWS CLI v2 Linux installer command ([abcee54](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/abcee549b56a2d766f343a39ebe87344f4e3d003))
* Use port 0 for dynamic port binding ([76f404e](https://github.com/ManotLuijiu/auto-affiliate-agents/commit/76f404e8b168be7801fc4116bdc05473fe219099))

## [1.0.0] - 2026-07-25

### Added

- Initial release of AMOS Companion
- Multi-platform support: macOS, Windows, Linux
- ADB device connection management
- Screen mirroring via scrcpy
- Device agent installation and management
- Cross-platform builds (Apple Silicon, Intel, Windows, Linux)

### Platforms

- macOS Apple Silicon (aarch64)
- macOS Intel (x86_64)
- Windows (x64)
- Linux (amd64 AppImage)

[1.0.0]: https://github.com/ManotLuijiu/amos-companion/releases/tag/companion/v1.0.0
