# 10. Assets

## 10.1 Art direction

The look is stylized low-poly with soft, rich light. Shapes are simple and easy to read. Light, fog and color make the mystical mood.

| Element | Rule |
|---|---|
| Shapes | Chunky silhouettes. Shapes stay clear at a distance. |
| Materials | Mostly flat colors and small gradient textures. No photo textures. |
| Palette | Dusk violet, ember orange, moss green, river blue and starlight cyan. Each region has a variation of the palette. |
| Light | By day, a warm sun and cool shadows. At dusk, deep violet and cyan. At night, strong contrast between light sources and dark areas. |
| Atmosphere | Layers of fog, light shafts, floating motes, fireflies and plants that glow. |
| Motion | Wind moves grass, leaves, cloth and water at all times. |

## 10.2 Sources

The main source is the CC0 asset family by Quaternius. These packs share one style, and the character packs share one rig.

| Pack | Use |
|---|---|
| Universal Base Characters | Base bodies and heads |
| Universal Animation Library 1 and 2 | Humanoid animation clips |
| Modular Character Outfits – Fantasy | Outfits |
| Medieval Village MegaKit | Buildings and village parts |
| Fantasy Props MegaKit | Props, tools and weapons |
| Stylized Nature MegaKit | Trees, plants and rocks |

The project uses only the free tiers of these packs. Other CC0 or CC-BY-4.0 sources, and small edits in Blender, fill the gaps. All additions must follow the style rules in 10.4.

Creature assets for wildlife and tamed creatures are an open question (see 12.10).

## 10.3 License rules

| License | Status |
|---|---|
| CC0-1.0 | Approved. This is the first choice. |
| CC-BY-4.0 | Approved. The credits screen and the license record must give the attribution. |
| OFL-1.1 | Approved for font files only. |
| All other licenses | Not approved. This includes all NC, ND and SA licenses, the GPL, sources with unclear terms and Mixamo files. |

Each asset has a license record with its source URL, its author, its license and the date of the download. Each asset archive contains a `REUSE.toml` file with these records. CI downloads the archives and runs `reuse lint` on them.

## 10.4 Style rules

| Rule | Value |
|---|---|
| Scale | 1 unit is 1 m. An adult human is 1.6 m to 1.9 m tall. |
| Axes | +Y is up and −Z is forward (Bevy convention). The import step converts the axes of glTF files. |
| Pivot | At the base of the object, at ground level. |
| Triangle budget | Player character 15,000. NPC 8,000. Creature 6,000. Prop 1,500. Tree 5,000 at LOD 0. Building 10,000. |
| Texture size | 1024 × 1024 or less. Most props use a shared palette texture. |
| Materials | Only the Vespera stylized material. |
| Rigs | Each humanoid uses the universal humanoid rig (see chapter 4). |
| Names | Lowercase snake case for files and nodes, for example `tree_oak_large_01`. |

## 10.5 Asset store

The git repository is text-only. All binary files are in the asset store.

- The asset store is the release area of a separate public GitHub repository. The proposed name is `vespera-assets`.
- Each release is a set of archives with a version. A release never changes after its publication.
- The file `assets.toml` in the main repository lists each archive: the name, the version, the URL, the SHA-256 hash, the size and the license.
- `cargo xtask fetch-assets` downloads the archives, verifies each hash and extracts the archives to a local cache.
- The asset store also contains the layer maps of the world compiler and the golden screenshots of the visual tests.
- To add an archive, a maintainer uploads it with `gh release upload`. Then the maintainer changes `assets.toml` in a commit.

## 10.6 Asset pipeline

1. Source. The original packs and the Blender files, in source archives.
2. Import. `cargo xtask import-assets` does these steps:
   - It converts the axes and the scale.
   - It gives the bones their Vespera joint names.
   - It applies the palette and the tint masks.
   - It makes LOD meshes with meshoptimizer.
   - It compresses the textures to BCn with mipmaps.
3. Runtime bundle. The import step writes the processed files to a runtime archive. The client and the server load only runtime files.

The import step is deterministic. Asset tests check the output: the triangle budgets, the texture sizes, the joint names and a license record for each file.

## 10.7 Audio

| Item | Default |
|---|---|
| Engine | `bevy_kira_audio` |
| Spatial audio | Yes, for all sounds in the world |
| Ambience | Layers for each biome, each time of day and each weather state |
| Vesper Hour | A sound motif that starts the Vesper Hour |
| Sound effects | CC0 and CC-BY-4.0 sources |
| Music | An open question (see 12.10) |

## 10.8 Fonts

Fonts use the OFL-1.1 license or CC0. The game uses one display font for titles and one text font for all other text.
