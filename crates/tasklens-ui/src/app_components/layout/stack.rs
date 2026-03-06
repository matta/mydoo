use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[css_module("/src/app_components/layout/stack.css")]
struct Styles;

/// Controls vertical spacing between children in [`Stack`].
///
/// Each variant maps to one app spacing token in `stack.css`:
/// `--app_spacing_xs|sm|md|lg|xl`.
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub(crate) enum StackGap {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

impl StackGap {
    fn class_name(self) -> &'static str {
        match self {
            Self::Xs => Styles::gap_xs.inner,
            Self::Sm => Styles::gap_sm.inner,
            Self::Md => Styles::gap_md.inner,
            Self::Lg => Styles::gap_lg.inner,
            Self::Xl => Styles::gap_xl.inner,
        }
    }
}

/// Controls cross-axis alignment for children in [`Stack`].
///
/// `Stretch` keeps children full-width by default, while the remaining
/// variants map to `align-items: flex-start|center|flex-end`.
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub(crate) enum StackAlign {
    #[default]
    Stretch,
    Start,
    Center,
    End,
}

impl StackAlign {
    fn class_name(self) -> &'static str {
        match self {
            Self::Stretch => Styles::align_stretch.inner,
            Self::Start => Styles::align_start.inner,
            Self::Center => Styles::align_center.inner,
            Self::End => Styles::align_end.inner,
        }
    }
}

#[component]
pub(crate) fn Stack(
    #[props(default)] gap: StackGap,
    #[props(default)] align: StackAlign,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: Styles::stack,
        class: gap.class_name(),
        class: align.class_name(),
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gap_enum_maps_to_expected_classes() {
        assert_eq!(StackGap::Xs.class_name(), Styles::gap_xs.inner);
        assert_eq!(StackGap::Sm.class_name(), Styles::gap_sm.inner);
        assert_eq!(StackGap::Md.class_name(), Styles::gap_md.inner);
        assert_eq!(StackGap::Lg.class_name(), Styles::gap_lg.inner);
        assert_eq!(StackGap::Xl.class_name(), Styles::gap_xl.inner);
    }

    #[test]
    fn align_enum_maps_to_expected_classes() {
        assert_eq!(
            StackAlign::Stretch.class_name(),
            Styles::align_stretch.inner
        );
        assert_eq!(StackAlign::Start.class_name(), Styles::align_start.inner);
        assert_eq!(StackAlign::Center.class_name(), Styles::align_center.inner);
        assert_eq!(StackAlign::End.class_name(), Styles::align_end.inner);
    }

    #[test]
    fn defaults_map_to_md_gap_and_stretch_alignment() {
        assert_eq!(StackGap::default().class_name(), Styles::gap_md.inner);
        assert_eq!(
            StackAlign::default().class_name(),
            Styles::align_stretch.inner
        );
    }

    #[test]
    fn stack_renders_in_virtual_dom() {
        fn app() -> Element {
            rsx! {
                Stack {
                    div { "Child" }
                }
            }
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
    }
}
