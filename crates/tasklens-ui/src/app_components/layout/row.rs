use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[css_module("/src/app_components/layout/row.css")]
struct Styles;

/// Controls horizontal spacing between children in [`Row`].
///
/// Each variant maps to one app spacing token in `row.css`.
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub(crate) enum RowGap {
    None,
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

impl RowGap {
    fn class_name(self) -> &'static str {
        match self {
            Self::None => Styles::gap_none.inner,
            Self::Xs => Styles::gap_xs.inner,
            Self::Sm => Styles::gap_sm.inner,
            Self::Md => Styles::gap_md.inner,
            Self::Lg => Styles::gap_lg.inner,
            Self::Xl => Styles::gap_xl.inner,
        }
    }
}

/// Controls cross-axis alignment for children in [`Row`].
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub(crate) enum RowAlign {
    Start,
    #[default]
    Center,
    End,
    Stretch,
    Baseline,
}

impl RowAlign {
    fn class_name(self) -> &'static str {
        match self {
            Self::Start => Styles::align_start.inner,
            Self::Center => Styles::align_center.inner,
            Self::End => Styles::align_end.inner,
            Self::Stretch => Styles::align_stretch.inner,
            Self::Baseline => Styles::align_baseline.inner,
        }
    }
}

/// Controls main-axis distribution for children in [`Row`].
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub(crate) enum RowJustify {
    #[default]
    Start,
    Center,
    End,
    Between,
}

impl RowJustify {
    fn class_name(self) -> &'static str {
        match self {
            Self::Start => Styles::justify_start.inner,
            Self::Center => Styles::justify_center.inner,
            Self::End => Styles::justify_end.inner,
            Self::Between => Styles::justify_between.inner,
        }
    }
}

/// Controls wrapping behavior for children in [`Row`].
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub(crate) enum RowWrap {
    #[default]
    NoWrap,
    Wrap,
}

impl RowWrap {
    fn class_name(self) -> &'static str {
        match self {
            Self::NoWrap => Styles::wrap_nowrap.inner,
            Self::Wrap => Styles::wrap_wrap.inner,
        }
    }
}

#[component]
pub(crate) fn Row(
    #[props(default)] gap: RowGap,
    #[props(default)] align: RowAlign,
    #[props(default)] justify: RowJustify,
    #[props(default)] wrap: RowWrap,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: Styles::row,
        class: "{gap.class_name()} {align.class_name()} {justify.class_name()} {wrap.class_name()}",
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
    use pretty_assertions::assert_str_eq;

    #[test]
    fn gap_enum_maps_to_expected_classes() {
        assert_eq!(RowGap::None.class_name(), Styles::gap_none.inner);
        assert_eq!(RowGap::Xs.class_name(), Styles::gap_xs.inner);
        assert_eq!(RowGap::Sm.class_name(), Styles::gap_sm.inner);
        assert_eq!(RowGap::Md.class_name(), Styles::gap_md.inner);
        assert_eq!(RowGap::Lg.class_name(), Styles::gap_lg.inner);
        assert_eq!(RowGap::Xl.class_name(), Styles::gap_xl.inner);
    }

    #[test]
    fn align_enum_maps_to_expected_classes() {
        assert_eq!(RowAlign::Start.class_name(), Styles::align_start.inner);
        assert_eq!(RowAlign::Center.class_name(), Styles::align_center.inner);
        assert_eq!(RowAlign::End.class_name(), Styles::align_end.inner);
        assert_eq!(RowAlign::Stretch.class_name(), Styles::align_stretch.inner);
        assert_eq!(
            RowAlign::Baseline.class_name(),
            Styles::align_baseline.inner
        );
    }

    #[test]
    fn justify_enum_maps_to_expected_classes() {
        assert_eq!(RowJustify::Start.class_name(), Styles::justify_start.inner);
        assert_eq!(
            RowJustify::Center.class_name(),
            Styles::justify_center.inner
        );
        assert_eq!(RowJustify::End.class_name(), Styles::justify_end.inner);
        assert_eq!(
            RowJustify::Between.class_name(),
            Styles::justify_between.inner
        );
    }

    #[test]
    fn wrap_enum_maps_to_expected_classes() {
        assert_eq!(RowWrap::NoWrap.class_name(), Styles::wrap_nowrap.inner);
        assert_eq!(RowWrap::Wrap.class_name(), Styles::wrap_wrap.inner);
    }

    #[test]
    fn defaults_map_to_md_gap_center_alignment_start_justification_and_no_wrap() {
        assert_eq!(RowGap::default().class_name(), Styles::gap_md.inner);
        assert_eq!(RowAlign::default().class_name(), Styles::align_center.inner);
        assert_eq!(
            RowJustify::default().class_name(),
            Styles::justify_start.inner
        );
        assert_eq!(RowWrap::default().class_name(), Styles::wrap_nowrap.inner);
    }

    #[test]
    fn row_renders_in_virtual_dom() {
        fn app() -> Element {
            rsx! {
                Row {
                    div { "Child" }
                }
            }
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
    }

    #[test]
    fn row_spread_attributes_match_equivalent_div() {
        fn row_component() -> Element {
            rsx! {
                Row {
                    id: "row-root",
                    class: "caller_class",
                    gap: RowGap::Lg,
                    align: RowAlign::Baseline,
                    justify: RowJustify::End,
                    wrap: RowWrap::Wrap,
                    "Child"
                }
            }
        }

        fn equivalent_div() -> Element {
            rsx! {
                div {
                    class: "{Styles::row} {Styles::gap_lg.inner} {Styles::align_baseline.inner} {Styles::justify_end.inner} {Styles::wrap_wrap.inner} caller_class",
                    id: "row-root",
                    "Child"
                }
            }
        }

        assert_component_rsx_eq(row_component, equivalent_div);
    }

    fn assert_component_rsx_eq(first: fn() -> Element, second: fn() -> Element) {
        let first = render_component(first);
        let second = render_component(second);
        assert_str_eq!(first, second);
    }

    fn render_component(component: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(component);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}
