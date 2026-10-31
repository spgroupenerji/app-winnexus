use super::*;
use gpui_kit::component::sidebar::{Sidebar, SidebarGroup, SidebarItem};
use gpui_kit::component::Collapsible;
use std::rc::Rc;

const YAN_MENU_GENISLIGI_PX: f32 = 240.;

type TiklaIsleyici = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(Clone)]
struct YanMenuOgesi {
    sayfa: Sayfa,
    aktif: bool,
    tikla: TiklaIsleyici,
}

impl Collapsible for YanMenuOgesi {
    fn is_collapsed(&self) -> bool {
        false
    }

    fn collapsed(self, _collapsed: bool) -> Self {
        self
    }
}

impl SidebarItem for YanMenuOgesi {
    fn render(
        self,
        id: impl Into<ElementId>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> impl IntoElement {
        let icerik = h_flex()
            .w_full()
            .gap_2()
            .items_center()
            .justify_start()
            .child(Icon::new(self.sayfa.ikon()).with_size(gpui_kit::component::Size::Small))
            .child(div().text_sm().child(self.sayfa.baslik()));
        let buton = Button::new(id)
            .with_size(gpui_kit::component::Size::Small)
            .w_full()
            .accessibility_label(self.sayfa.baslik())
            .child(icerik);
        let buton = if self.aktif {
            buton.primary()
        } else {
            buton.ghost()
        };
        let tikla = self.tikla;
        buton.on_click(move |olay, window, cx| tikla(olay, window, cx))
    }
}

impl WinNexusApp {
    pub(super) fn yan_menu(&self, bolum: Bolum, cx: &mut Context<Self>) -> AnyElement {
        let ogeler = bolum.sayfalar().iter().map(|sayfa| {
            let sayfa = *sayfa;
            YanMenuOgesi {
                sayfa,
                aktif: self.sayfa == sayfa,
                tikla: Rc::new(cx.listener(move |this, _, window, cx| {
                    this.sayfaya_git(sayfa, window, cx);
                })),
            }
        });
        Sidebar::new("yan-menu")
            .w(px(YAN_MENU_GENISLIGI_PX))
            .collapsible(false)
            .child(SidebarGroup::new(bolum.baslik()).children(ogeler))
            .into_any_element()
    }
}
