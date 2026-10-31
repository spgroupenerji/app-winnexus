use super::*;
use gpui_kit::component::sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem};

impl WinNexusApp {
    pub(super) fn yan_menu(&self, bolum: Bolum, cx: &mut Context<Self>) -> AnyElement {
        let ogeler = bolum.sayfalar().iter().map(|sayfa| {
            let sayfa = *sayfa;
            SidebarMenuItem::new(sayfa.baslik())
                .icon(Icon::new(sayfa.ikon()).with_size(gpui_kit::component::Size::Small))
                .active(self.sayfa == sayfa)
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.sayfaya_git(sayfa, window, cx);
                }))
        });
        Sidebar::new("yan-menu")
            .w(px(240.))
            .collapsible(false)
            .child(SidebarGroup::new(bolum.baslik()).child(SidebarMenu::new().children(ogeler)))
            .into_any_element()
    }
}
