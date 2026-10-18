// 嵌入 exe 图标与版本信息（资源管理器/任务管理器里显示）。
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/mousetroke.ico")
            // 32512：窗口类默认图标的资源号，设置界面的标题栏/任务栏按钮取这个
            .set_icon_with_id("assets/mousetroke.ico", "32512")
            .set("FileDescription", "Mousetroke mouse gestures")
            .set("ProductName", "Mousetroke")
            // 设置界面也在这个 exe 里：启用新版通用控件（菜单/对话框外观）
            .set_manifest(MANIFEST);
        if let Err(e) = res.compile() {
            println!("cargo:warning=图标资源嵌入失败: {e}");
        }
    }
}

const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0"
        processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" />
    </dependentAssembly>
  </dependency>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
    </application>
  </compatibility>
</assembly>
"#;
