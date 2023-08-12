use godot::prelude::*;

struct KataruExtension;

#[gdextension]
unsafe impl ExtensionLibrary for KataruExtension {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_load() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
