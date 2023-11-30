use std::sync::RwLock;
use rspack_core::{
    Plugin,
    PluginContext,
    NormalModuleBeforeResolveArgs,
    PluginNormalModuleFactoryBeforeResolveOutput,
};

#[derive(Debug)]
pub struct SpecilizeModuleNamePlugin {
    // this value will auto-increase in concurrence situation
    uid: RwLock<i32>,
    target_module_names: Vec<String>,
}

impl SpecilizeModuleNamePlugin {
    pub fn new(target_module_names: Option<Vec<String>>) -> Self {
        Self {
            uid: RwLock::new(0),
            target_module_names: match target_module_names {
                Some(value) => value,
                None => vec![],
            },
        }
    }

    fn increase_uid(&self) -> i32 {
        let mut cur_id = self.uid.write().unwrap();
        *cur_id += 1;
        *cur_id
    }
}

#[async_trait::async_trait]
impl Plugin for SpecilizeModuleNamePlugin {
    fn name(&self) -> &'static str {
        "SpecilizeModuleNamePlugin"
    }

    async fn before_resolve(
        &self,
        _ctx: PluginContext,
        _args: &mut NormalModuleBeforeResolveArgs
    ) -> PluginNormalModuleFactoryBeforeResolveOutput {
        if self.target_module_names.is_empty() {
            return Ok(None);
        }
        for name in &self.target_module_names {
            if _args.request.contains(name) {
                let uid = self.increase_uid().to_string();
                _args.request.push_str("?id=");
                _args.request.push_str(&uid);
                break;
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_instance_without_argument() {
        let specilize_module_name_plugin = SpecilizeModuleNamePlugin::new(None);

        assert_eq!(*specilize_module_name_plugin.uid.read().unwrap(), 0);
        assert_eq!(
            specilize_module_name_plugin.target_module_names.as_slice(),
            &vec![] as &[&'static str]
        );
    }
    #[test]
    fn test_create_instance_with_argument() {
        let target = vec!["a".to_string(), "b".to_string()];
        let specilize_module_name_plugin = SpecilizeModuleNamePlugin::new(Some(target.clone()));

        assert_eq!(*specilize_module_name_plugin.uid.read().unwrap(), 0);
        assert_eq!(specilize_module_name_plugin.target_module_names.as_slice(), &target);
    }

    #[test]
    fn test_increase_uid_method() {
        let specilize_module_name_plugin = SpecilizeModuleNamePlugin::new(None);
        specilize_module_name_plugin.increase_uid();
        let result_1 = specilize_module_name_plugin.increase_uid();
        assert_eq!(result_1, 2);
        assert_eq!(*specilize_module_name_plugin.uid.read().unwrap(), 2);
    }

    #[test]
    fn test_name_method() {
        let specilize_module_name_plugin = SpecilizeModuleNamePlugin::new(None);
        assert_eq!(specilize_module_name_plugin.name(), "SpecilizeModuleNamePlugin");
    }
}
