"""dmgbuild 1.6.5 布局；兼容 Mac mini 的 Python 3.9。"""
format = "UDZO"
filesystem = "HFS+"
files = [defines["app"]]
symlinks = {"Applications": "/Applications"}
# dmgbuild 1.6.5 默认 lookForHiDPI=True；自动发现同目录的 @2x，
# 用 tiffutil -cathidpicheck 合成镜像内唯一的 .background.tiff，无需另复制 PNG 目录。
# 源码：https://github.com/dmgbuild/dmgbuild/blob/v1.6.5/src/dmgbuild/core.py
background = defines["art"] + "/background.png"
window_rect = ((120, 120), (640, 400))
icon_size = 128
text_size = 14
icon_locations = {"Mousetroke.app": (160, 200), "Applications": (480, 200)}
default_view = "icon-view"
arrange_by = None
show_toolbar = False
show_sidebar = False
show_status_bar = False
show_pathbar = False
show_tab_view = False
include_icon_view_settings = True
include_list_view_settings = False
hide_extensions = ["Mousetroke.app"]
