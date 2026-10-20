use mousetroke_core::schema::{Action, MouseButton};
use mousetroke_core::vars::Vars;

/// 手势开始时锁定的目标数据；窗口标识由宿主解释，不携带平台句柄类型。
#[derive(Debug, Clone)]
pub struct Target {
    /// 平台窗口标识，仅原样回传宿主与写入 MS_WID。
    pub wid: usize,
    pub pid: u32,
    /// 目标程序完整路径（可能取不到）。
    pub exe: Option<String>,
    pub is_desktop: bool,
    /// 目标窗口类别，用于捕获日志。
    pub class: String,
}

/// 轨迹/标签绘制面。全部方法在引擎线程调用；输入回调内须快速返回，
/// 不得等待、无界分配或 panic；绘制与标签实现沿用宿主的有界缓冲策略。
pub trait Canvas {
    /// 设置轨迹颜色；配置更新或捕获开始/结束时调用。
    fn set_color(&mut self, color: &str);
    /// 设置轨迹宽度；配置更新时调用。
    fn set_width(&mut self, width_px: f64);
    /// 在屏幕坐标开始一次绘制；输入回调调用，须快速返回。
    fn start_session(&mut self, x: i32, y: i32);
    /// 追加屏幕坐标轨迹点；高频输入回调调用，不得无界扩容。
    fn line_to(&mut self, x: i32, y: i32);
    /// 立即清除本次轨迹；输入回调或计时器调用。
    fn finish(&mut self);
    /// 开始轨迹淡出；输入回调调用，不得等待动画完成。
    fn fade_out(&mut self);
    /// 推进一帧淡出；引擎线程的绘制定时器调用。
    fn tick_fade(&mut self);
    /// 在屏幕坐标绘制标签；输入回调或提示定时器调用，须快速返回。
    fn draw_label(&mut self, text: &str, x: i32, y: i32, color: &str);
    /// 清除标签；输入回调或计时器调用。
    fn clear_label(&mut self);
    /// 返回缓存的虚拟屏矩形 (x, y, w, h)；输入回调内频繁查询。
    fn virtual_rect(&self) -> (i32, i32, i32, i32);
}

/// 引擎对系统的查询与副作用，全部从引擎线程调用。输入回调内的方法
/// 必须快速返回，不得等待、无界分配或 panic；补发与动作执行必须异步，避免重入钩子。
pub trait Host {
    /// 查询屏幕坐标下的目标；输入回调调用，须快速返回。
    fn target_at(&self, x: i32, y: i32) -> Target;
    /// 判断设置界面/系统外壳等应直接放行的目标；按键捕获前调用。
    fn ignore_target(&self, target: &Target) -> bool;
    /// 查询目标标题以构建变量；捕获开始的输入回调调用，须快速返回。
    fn window_title(&self, wid: usize) -> String;
    /// 查询前台是否全屏；输入回调调用，须快速返回。
    fn foreground_fullscreen(&self) -> bool;
    /// 异步补发触发键：click 为按下+抬起，否则只补发按下；输入回调不得等待。
    fn replay(&self, btn: MouseButton, click: bool);
    /// 异步执行动作；输入回调只投递，实际注入、脚本、菜单等在平台工作线程执行。
    fn execute(&self, action: Action, target: Option<Target>, vars: Vars, pos: (i32, i32));
    /// 返回所在显示器工作区内的标签中心点；输入回调或提示定时器调用，须快速返回。
    fn label_anchor(&self, x: i32, y: i32) -> (i32, i32);
    /// 创建绘制面；仅启动与显示器布局变化时调用，不在输入回调内重建。
    fn new_canvas(&self, color: &str, width: f64) -> Option<Box<dyn Canvas>>;
    /// 投递固定大小校准样本；输入回调调用，队列满时丢弃，不得等待或写盘。
    fn shape_sample(&self, encoded: Vec<i32>);
    /// 查询当前光标；仅提示定时器调用，须快速返回。
    fn cursor_pos(&self) -> (i32, i32);
    /// 查询系统首选语言；仅启动/配置热加载调用。
    fn system_langs(&self) -> Vec<String>;
    /// 语言变化后刷新平台菜单；仅启动/配置热加载调用。
    fn refresh_texts(&self);
    /// 空闲时归还工作集；仅提示定时器调用，默认无需处理。
    fn trim_memory(&self) {}
}
