use sailfish::TemplateSimple;

#[derive(Debug, serde::Serialize, serde::Deserialize, TemplateSimple)]
#[template(path = "hello.stpl")]
pub struct TestTemplate {
    name: String,
    first_name: String,
}

impl TestTemplate {
    pub fn new(name: String, first_name: String) -> Self {
        Self { name, first_name }
    }
}

#[cfg(not(all(target_arch = "wasm32")))]
impl TestTemplate {
    pub fn render_once(&self) {
        let mut lock = shared::WASM_INSTANCE.lock().unwrap();
        let instance = lock.get_mut("guest");

        if let Some(instance) = instance {
            println!("calling guest...");
            let result = instance.call::<shared::Postcard<TestTemplate>, String>(
                "render_test_template",
                shared::Postcard(TestTemplate::new("Builder".into(), "Bob".into())),
            );
            println!("{:?}", result);
        } else {
            println!("instance not found");
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[extism_pdk::plugin_fn]
pub fn render_test_template(
    template: shared::Postcard<TestTemplate>,
) -> extism_pdk::FnResult<String> {
    let template = template.into_inner();

    template.render_once().map_err(Into::into)
}
