use std::path::{Path, PathBuf};

use bevy::render::settings::{Backends, Dx12Compiler};

pub const DXC_LIBRARY: &str = "dxcompiler.dll";

#[must_use]
pub fn render_backends(from_environment: Option<Backends>, windows: bool) -> Backends {
    from_environment.unwrap_or(if windows {
        Backends::DX12
    } else {
        Backends::all()
    })
}

#[must_use]
pub fn shader_compiler(
    from_environment: Option<Dx12Compiler>,
    dxc_library: Option<PathBuf>,
) -> Dx12Compiler {
    from_environment.unwrap_or_else(|| {
        dxc_library.map_or(Dx12Compiler::Fxc, |library| Dx12Compiler::DynamicDxc {
            dxc_path: library.to_string_lossy().into_owned(),
        })
    })
}

#[must_use]
pub fn dxc_next_to(executable: &Path) -> Option<PathBuf> {
    let library = executable.parent()?.join(DXC_LIBRARY);
    library.is_file().then_some(library)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use bevy::render::settings::{Backends, Dx12Compiler};

    use super::{DXC_LIBRARY, dxc_next_to, render_backends, shader_compiler};

    #[test]
    fn windows_uses_direct3d_12_by_default() {
        assert_eq!(render_backends(None, true), Backends::DX12);
    }

    #[test]
    fn other_systems_keep_all_backends() {
        assert_eq!(render_backends(None, false), Backends::all());
    }

    #[test]
    fn the_wgpu_backend_variable_wins_on_windows() {
        assert_eq!(
            render_backends(Some(Backends::VULKAN), true),
            Backends::VULKAN
        );
    }

    #[test]
    fn a_dxc_library_next_to_the_game_selects_dxc_with_its_full_path() {
        let library = PathBuf::from("C:/Games/Vespera/dxcompiler.dll");

        let compiler = shader_compiler(None, Some(library));

        assert!(
            matches!(compiler, Dx12Compiler::DynamicDxc { dxc_path } if dxc_path == "C:/Games/Vespera/dxcompiler.dll")
        );
    }

    #[test]
    fn without_a_dxc_library_direct3d_12_uses_fxc() {
        assert!(matches!(shader_compiler(None, None), Dx12Compiler::Fxc));
    }

    #[test]
    fn the_wgpu_dx12_compiler_variable_wins() {
        let library = PathBuf::from("dxcompiler.dll");

        let compiler = shader_compiler(Some(Dx12Compiler::Fxc), Some(library));

        assert!(matches!(compiler, Dx12Compiler::Fxc));
    }

    #[test]
    fn the_dxc_library_is_found_next_to_the_executable() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join(DXC_LIBRARY), "library").unwrap();

        let found = dxc_next_to(&directory.path().join("vespera.exe"));

        assert_eq!(found, Some(directory.path().join(DXC_LIBRARY)));
    }

    #[test]
    fn a_missing_dxc_library_is_not_found() {
        let directory = tempfile::tempdir().unwrap();

        assert_eq!(dxc_next_to(&directory.path().join("vespera.exe")), None);
    }
}
