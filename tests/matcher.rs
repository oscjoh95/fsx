mod levenshtein_tests {
    use fsx::matcher::{Levenshtein, Matcher};
    use fsx::test_utils::approx_eq;

    fn matcher() -> Levenshtein {
        Levenshtein::new()
    }

    #[test]
    fn exact_match_scores_one() {
        let m = matcher();
        assert!(approx_eq(m.score("file", "file"), 1.0));
    }

    #[test]
    fn empty_vs_empty_scores_one() {
        let m = matcher();
        assert!(approx_eq(m.score("", ""), 1.0));
    }

    #[test]
    fn empty_vs_non_empty_scores_zero() {
        let m = matcher();
        assert!(approx_eq(m.score("", "abc"), 0.0));
        assert!(approx_eq(m.score("abc", ""), 0.0));
    }

    #[test]
    fn single_edit_reduces_score() {
        let m = matcher();
        let exact = m.score("file", "file");
        let typo = m.score("file", "fiel");
        assert!(typo < exact);
    }

    #[test]
    fn known_example_kitten_sitting() {
        let m = matcher();

        // distance = 3, max_len = 7
        let expected = 1.0 - (3.0 / 7.0);

        assert!(approx_eq(m.score("kitten", "sitting"), expected));
    }

    #[test]
    fn closer_match_scores_higher_than_distant_match() {
        let m = matcher();

        let close = m.score("report", "reprot");
        let distant = m.score("report", "banana");

        assert!(close > distant);
    }

    #[test]
    fn score_is_always_in_range() {
        let m = matcher();

        let samples = [
            ("file", "file"),
            ("file", "fiel"),
            ("file", "files"),
            ("file", ""),
            ("", "file"),
            ("kitten", "sitting"),
            ("abc", "xyz"),
        ];

        for (q, c) in samples {
            let s = m.score(q, c);
            assert!(s >= 0.0 && s <= 1.0);
        }
    }
}
