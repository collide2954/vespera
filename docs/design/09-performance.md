# 9. Performance

## 9.1 Spec floor

| Part | Spec floor |
|---|---|
| Computer | Lenovo ThinkPad T590 |
| CPU | Intel Core i5-8265U or i5-8365U (4 cores, 8 threads) |
| GPU | Intel UHD Graphics 620 (integrated) |
| Memory | 12 GB, shared with the GPU |
| Display | 1920 × 1080 |
| Storage | SSD |

The reference benchmark computer is a T590 with an i5-8365U. It has more memory than the spec floor. Thus memory budgets must fit a computer with 12 GB.

On Windows, the client uses Direct3D 12. On the reference computer, the client crashed in the Intel Vulkan driver. The `WGPU_BACKEND` variable can select a different backend.

Direct3D 12 uses the DXC shader compiler from the file `dxcompiler.dll` next to the client. Without that file, the client uses the older FXC compiler and writes a warning. With FXC, the main render pass was approximately 30 times slower on the reference computer. The `WGPU_DX12_COMPILER` variable can select a different compiler.

## 9.2 Client targets

These targets are for the Low preset on the spec floor.

| Metric | Target |
|---|---|
| Frame rate | 60 FPS at 1080p output |
| Render scale | Dynamic, from 67% to 100% |
| 1% low frame rate in a crowded town with 100 characters | 30 FPS or more |
| Frames during chunk streaming | No frame longer than 33 ms |
| Time from launch to the world | 20 s or less |
| Process memory | 2.5 GB or less |
| GPU memory | 1.0 GB or less |
| Network download | 64 kbit/s typical, 256 kbit/s peak |
| Network upload | 16 kbit/s or less |

## 9.3 Frame budget

| Work | Thread | Budget |
|---|---|---|
| Game logic, prediction and network | Main | 2.0 ms |
| Animation and IK | Main, in parallel | 2.0 ms |
| UI | Main | 1.0 ms |
| Render extraction and preparation | Render | 5.0 ms |
| Shadow maps | GPU | 2.5 ms |
| Opaque geometry and terrain | GPU | 6.0 ms |
| Foliage | GPU | 2.0 ms |
| Sky, fog and water | GPU | 1.5 ms |
| Post-processing and upscale | GPU | 1.5 ms |
| UI | GPU | 0.5 ms |

The GPU budget has a total of 14 ms. The remaining 2.7 ms is a reserve.

## 9.4 Server targets

| Metric | Target |
|---|---|
| Players for each shard | 1,000 |
| NPCs and creatures for each shard | 10,000 |
| Rules tick | 20 Hz, with 25 ms p99 or less on 8 cores |
| Movement step | 60 Hz |
| Network upload for each shard | 100 Mbit/s or less |

## 9.5 Render rules for the Low preset

- Use Bevy forward rendering with the Vespera material. On Low, the material uses few texture samples and a simple light model.
- Do not use MSAA. Use SMAA or FXAA, then contrast adaptive sharpening (CAS).
- Render the 3D scene at the dynamic render scale. Render the UI at the full output resolution.
- The scene camera renders into an image at the render scale and applies FXAA, then CAS. The screen camera shows the image in the window with bilinear filtering. The UI renders on the screen camera.
- Do not apply CAS at the output resolution on Low. On the reference computer, CAS at 1920 × 1080 took 2.1 to 2.7 ms of each frame.
- Do not use the main pass resolution override of Bevy. The light clusters of Bevy do not use the override. Thus, point lights and spot lights go into incorrect clusters.
- On Low, the scene camera does not use indirect draws. The indirect draws of Bevy add many preparation steps on the render thread in each frame. On the reference computer, direct draws gave the same image at an approximately 20% higher frame rate. Do not turn off the GPU instance buffer builder of Bevy for this purpose. With Bevy 0.20.1, the main pass then did not draw the meshes.
- On Low, cluster the lights on the CPU. GPU clustering adds a compute pass, and work on the render thread, in each frame. With few point lights, CPU clustering takes almost no time.
- Use 2 shadow cascades at 1024 × 1024, up to 60 m, with the hardware 2 × 2 shadow filter. Contact shadows are off.
- Use distance fog and height fog. Volumetric fog is only for the High and Ultra presets.
- Update the sky at a low rate, not in each frame.
- Use instanced grass with wind in the vertex shader. On Low, grass fades out at 40 m.
- Use LOD meshes for trees, and impostors at long distances.
- Use GPU-compressed textures (BCn) with mipmaps. The maximum texture size on Low is 1024 × 1024.
- Use texture arrays and atlases to decrease bind changes.
- Use visibility ranges for hierarchical LOD.
- Keep 1.5 million visible triangles or less, and 1,500 draw calls or less.

## 9.6 Quality presets

| Setting | Low | Medium | High | Ultra |
|---|---|---|---|---|
| Render scale | 67% to 100%, dynamic | 85% to 100%, dynamic | 100% | 100% |
| Shadow cascades | 2 × 1024 | 3 × 2048 | 4 × 2048 | 4 × 4096 |
| Grass distance | 40 m | 70 m | 110 m | 160 m |
| View distance | 600 m | 1,000 m | 1,500 m | 2,000 m |
| Volumetric fog | Off | Off | On | On |
| Contact shadows | Off | Off | On | On |
| Screen-space reflections | Off | Off | Water only | On |
| Bloom | Low | Medium | High | High |

Each preset is a data file. At the first launch, the client selects a preset from the GPU name and a short benchmark.

The dynamic render scale has 5 steps from the minimum to the maximum of the preset. A controller compares the mean frame time of each 30 frames with the frame budget:

- The frame budget is 1/60 s. With vsync on a display slower than 60 Hz, the budget is one refresh interval of the display.
- A mean of more than 110% of the budget lowers the scale by one step.
- After 4 windows with a mean of less than 105% of the budget, the controller raises the scale by one step.
- With vsync, the frame time does not show the free GPU time. Thus, each raise is a test. A slow window in the 4 windows after a raise stops the test.
- After each failed test, the controller waits two times longer before the next test, up to 64 windows.

## 9.7 Measurement

- `cargo xtask bench` starts the client in benchmark mode. The camera follows a fixed path through the test valley at dusk, with 100 bot characters.
- The client starts in benchmark mode when the `VESPERA_BENCHMARK_SECONDS` variable is set. It starts to record frames when the client controls its character.
- In benchmark mode, the client uses the full screen without vsync. Thus, the frame times show the work of the computer and not the refresh rate of the display.
- In benchmark mode, the client updates without pauses, also when its window does not have the focus. Without the focus, Bevy usually waits until 1/60 s after the start of each frame.
- In benchmark mode, GPU timers measure each render pass. The timers decrease the frame rate by approximately 5%.
- The reference computer becomes slower when it is hot. Thus, compare two changes with runs one after the other, not with old results.
- Measure on mains power. On battery, the reference computer limits the GPU clock.
- The benchmark records these values:
  - The frame times: p50, p95, p99 and 1% low.
  - The GPU time of each render pass.
  - The process memory and the GPU memory.
- The benchmark writes a JSON report to the `benchmarks/` folder.
- Run the benchmark on the reference computer at the end of each milestone, and after each large render change.
- In CI, criterion benchmarks measure the hot paths: the rules tick with 1,000 players, IK for 100 characters and the exchange order matches. A slowdown of more than 10% causes a warning.
