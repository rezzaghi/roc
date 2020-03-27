#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate indoc;
extern crate bumpalo;
extern crate roc_reporting;

mod helpers;

#[cfg(test)]
mod test_report {
    use crate::helpers::test_home;
    use roc_module::symbol::{Interns, ModuleId};
    use roc_reporting::report::{
        can_problem, em_text, plain_text, url, Report, ReportText, BLUE_CODE, BOLD_CODE, CYAN_CODE,
        GREEN_CODE, MAGENTA_CODE, RED_CODE, RESET_CODE, TEST_PALETTE, UNDERLINE_CODE, WHITE_CODE,
        YELLOW_CODE,
    };
    use roc_types::pretty_print::name_all_type_vars;
    use roc_types::subs::Subs;
    use roc_types::types;
    use std::path::PathBuf;
    // use roc_region::all;
    use crate::helpers::{can_expr, infer_expr, CanExprOut};
    use roc_reporting::report::ReportText::{Batch, Region, Type, Value};
    use roc_types::subs::Content::{FlexVar, RigidVar, Structure};
    use roc_types::subs::FlatType::EmptyRecord;

    fn filename_from_string(str: &str) -> PathBuf {
        let mut filename = PathBuf::new();
        filename.push(str);

        return filename;
    }

    // use roc_problem::can;
    fn to_simple_report(text: ReportText) -> Report {
        Report {
            text: text,
            filename: filename_from_string(r"\code\proj\Main.roc"),
        }
    }

    fn infer_expr_help(
        expr_src: &str,
    ) -> (
        Vec<types::Problem>,
        Vec<roc_problem::can::Problem>,
        Subs,
        ModuleId,
        Interns,
    ) {
        let CanExprOut {
            output,
            var_store,
            var,
            constraint,
            home,
            interns,
            problems: can_problems,
            ..
        } = can_expr(expr_src);
        let mut subs = Subs::new(var_store.into());

        for (var, name) in output.introduced_variables.name_by_var {
            subs.rigid_var(var, name);
        }

        let mut unify_problems = Vec::new();
        let (_content, mut subs) = infer_expr(subs, &mut unify_problems, &constraint, var);

        name_all_type_vars(var, &mut subs);

        (unify_problems, can_problems, subs, home, interns)
    }

    fn report_renders_as_from_src(src: &str, report: Report, expected_rendering: &str) {
        let (_type_problems, _can_problems, mut subs, home, interns) = infer_expr_help(src);
        let mut buf: String = String::new();
        let src_lines: Vec<&str> = src.split('\n').collect();

        report
            .text
            .render_ci(&mut buf, &mut subs, home, &src_lines, &interns);

        assert_eq!(buf, expected_rendering);
    }

    fn human_readable(str: &str) -> String {
        return str
            .replace(RED_CODE, "<red>")
            .replace(WHITE_CODE, "<white>")
            .replace(BLUE_CODE, "<blue>")
            .replace(YELLOW_CODE, "<yellow>")
            .replace(GREEN_CODE, "<green>")
            .replace(CYAN_CODE, "<cyan>")
            .replace(MAGENTA_CODE, "<magenta>")
            .replace(RESET_CODE, "<reset>")
            .replace(BOLD_CODE, "<bold>")
            .replace(UNDERLINE_CODE, "<underline>");
    }

    fn report_renders_in_color_from_src(src: &str, report: Report, expected_rendering: &str) {
        let (_type_problems, _can_problems, mut subs, home, interns) = infer_expr_help(src);
        let mut buf: String = String::new();
        let src_lines: Vec<&str> = src.split('\n').collect();

        report.text.render_color_terminal(
            &mut buf,
            &mut subs,
            home,
            &src_lines,
            &interns,
            &TEST_PALETTE,
        );

        assert_eq!(human_readable(&buf), expected_rendering);
    }

    fn report_renders_in_color(report: Report, expected_rendering: &str) {
        report_renders_in_color_from_src(
            indoc!(
                r#"
                    x = 1
                    y = 2

                    x
                "#
            ),
            report,
            expected_rendering,
        )
    }

    fn report_renders_as(report: Report, expected_rendering: &str) {
        report_renders_as_from_src(
            indoc!(
                r#"
                    x = 1
                    y = 2

                    x
                "#
            ),
            report,
            expected_rendering,
        )
    }

    #[test]
    fn report_plain() {
        report_renders_as(to_simple_report(plain_text("y")), "y");
    }

    #[test]
    fn report_emphasized_text() {
        report_renders_as(to_simple_report(em_text("y")), "*y*");
    }

    #[test]
    fn report_url() {
        report_renders_as(
            to_simple_report(url("package.roc.org")),
            "<package.roc.org>",
        );
    }

    #[test]
    fn report_symbol() {
        let src: &str = indoc!(
            r#"
                x = 1
                y = 2

                x
            "#
        );

        let (_type_problems, _can_problems, mut subs, home, interns) = infer_expr_help(src);

        let mut buf = String::new();
        let src_lines: Vec<&str> = src.split('\n').collect();

        to_simple_report(Value(interns.symbol(test_home(), "x".into())))
            .text
            .render_ci(&mut buf, &mut subs, home, &src_lines, &interns);

        assert_eq!(buf, "x");
    }

    #[test]
    fn report_wildcard() {
        report_renders_as(to_simple_report(Type(FlexVar(None))), "*");
    }

    #[test]
    fn report_flex_var() {
        report_renders_as(to_simple_report(Type(FlexVar(Some("msg".into())))), "msg");
    }

    #[test]
    fn report_rigid_var() {
        report_renders_as(to_simple_report(Type(RigidVar("Str".into()))), "Str");
    }

    #[test]
    fn report_empty_record() {
        report_renders_as(to_simple_report(Type(Structure(EmptyRecord))), "{}");
    }

    #[test]
    fn report_batch_of_plain_text() {
        let mut report_texts = Vec::new();

        report_texts.push(plain_text("Wait a second. "));
        report_texts.push(plain_text("There is a problem here. -> "));
        report_texts.push(em_text("y"));

        report_renders_as(
            to_simple_report(Batch(report_texts)),
            "Wait a second. There is a problem here. -> *y*",
        );
    }

    #[test]
    fn report_unused_def() {
        let src: &str = indoc!(
            r#"
                x = 1
                y = 2

                x
            "#
        );

        let (_type_problems, can_problems, mut subs, home, interns) = infer_expr_help(src);

        let mut buf: String = String::new();
        let src_lines: Vec<&str> = src.split('\n').collect();

        match can_problems.first() {
            None => {}
            Some(problem) => {
                let report = can_problem(
                    filename_from_string(r"\code\proj\Main.roc"),
                    problem.clone(),
                );
                report
                    .text
                    .render_ci(&mut buf, &mut subs, home, &src_lines, &interns)
            }
        }

        assert_eq!(
            buf,
            indoc!(
                r#"
                y is not used anywhere in your code.

                2 ┆  y = 2

                If you didn't intend on using y then remove it so future readers of your code don't wonder why it is there."#
            )
        );
    }

    #[test]
    fn report_plain_text_color() {
        report_renders_in_color(to_simple_report(plain_text("y")), "<white>y<reset>");
    }

    #[test]
    fn report_em_text_color() {
        report_renders_in_color(to_simple_report(em_text("HELLO!")), "<bold>HELLO!<reset>");
    }

    #[test]
    fn report_url_color() {
        report_renders_in_color(
            to_simple_report(url("www.roc.com/blog")),
            "<underline>www.roc.com/blog<reset>",
        );
    }

    #[test]
    fn report_value_color() {
        let src: &str = indoc!(
            r#"
                activityIndicatorLarge = div

                view activityIndicatorLarge
            "#
        );

        let (_type_problems, _can_problems, mut subs, home, interns) = infer_expr_help(src);

        let mut buf = String::new();
        let src_lines: Vec<&str> = src.split('\n').collect();

        to_simple_report(Value(
            interns.symbol(test_home(), "activityIndicatorLarge".into()),
        ))
        .text
        .render_color_terminal(
            &mut buf,
            &mut subs,
            home,
            &src_lines,
            &interns,
            &TEST_PALETTE,
        );

        assert_eq!(human_readable(&buf), "<blue>activityIndicatorLarge<reset>");
    }

    #[test]
    fn report_wildcard_in_color() {
        report_renders_in_color(to_simple_report(Type(FlexVar(None))), "<yellow>*<reset>");
    }

    #[test]
    fn report_flex_var_in_color() {
        report_renders_in_color(
            to_simple_report(Type(FlexVar(Some("msg".into())))),
            "<yellow>msg<reset>",
        );
    }

    #[test]
    fn report_rigid_var_in_color() {
        report_renders_in_color(
            to_simple_report(Type(RigidVar("Str".into()))),
            "<yellow>Str<reset>",
        );
    }

    #[test]
    fn report_empty_record_in_color() {
        report_renders_in_color(
            to_simple_report(Type(Structure(EmptyRecord))),
            "<green>{}<reset>",
        );
    }

    #[test]
    fn report_batch_in_color() {
        let mut report_texts = Vec::new();

        report_texts.push(Type(RigidVar("List".into())));
        report_texts.push(plain_text(" "));
        report_texts.push(Type(Structure(EmptyRecord)));

        report_renders_in_color(
            to_simple_report(Batch(report_texts)),
            "<yellow>List<reset><white> <reset><green>{}<reset>",
        );
    }

    #[test]
    fn report_region_in_color() {
        report_renders_in_color_from_src(
            indoc!(
                r#"
                    isDisabled = \user -> user.isAdmin

                    theAdmin
                        |> isDabled
                "#
            ),
            to_simple_report(Region(roc_region::all::Region {
                start_line: 0,
                end_line: 3,
                start_col: 0,
                end_col: 0,
            })),
            indoc!(
                r#"
                    <cyan>1<reset><magenta> ┆<reset>  <white>isDisabled = \user -> user.isAdmin<reset>
                    <cyan>2<reset><magenta> ┆<reset>
                    <cyan>3<reset><magenta> ┆<reset>  <white>theAdmin<reset>
                    <cyan>4<reset><magenta> ┆<reset>  <white>    |> isDabled<reset>"#
            ),
        );
    }

    #[test]
    fn report_region() {
        report_renders_as_from_src(
            indoc!(
                r#"
                    x = 1
                    y = 2
                    f = \a -> a + 4

                    f x
                "#
            ),
            to_simple_report(Region(roc_region::all::Region {
                start_line: 1,
                end_line: 3,
                start_col: 0,
                end_col: 0,
            })),
            indoc!(
                r#"
                    2 ┆  y = 2
                    3 ┆  f = \a -> a + 4
                    4 ┆"#
            ),
        );
    }

    #[test]
    fn report_region_different_line_number_lengths() {
        report_renders_as_from_src(
            indoc!(
                r#"
                    x = 1








                    y = 2
                    f = \a -> a + 4

                    f x
                "#
            ),
            to_simple_report(Region(roc_region::all::Region {
                start_line: 8,
                end_line: 10,
                start_col: 0,
                end_col: 0,
            })),
            indoc!(
                r#"
                     9 ┆
                    10 ┆  y = 2
                    11 ┆  f = \a -> a + 4"#
            ),
        );
    }
}