use bevy::render::settings::Backends;

#[must_use]
pub fn render_backends(from_environment: Option<Backends>, windows: bool) -> Backends {
    from_environment.unwrap_or(if windows {
        Backends::DX12
    } else {
        Backends::all()
    })
}

#[cfg(test)]
mod tests {
    use bevy::render::settings::Backends;

    use super::render_backends;

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
}
