use std::io::Write;

#[derive(Debug, PartialEq)]
struct Mismatch {
    pub line_number_expected: usize,
    pub line_number_actual: usize,
    pub expected: Vec<Vec<u8>>,
    pub actual: Vec<Vec<u8>>,
    pub expected_missing_nl: bool,
    pub actual_missing_nl: bool,
}

impl Mismatch {
    fn new(line_number_expected: usize, line_number_actual: usize) -> Mismatch {
        Mismatch {
            line_number_expected,
            line_number_actual,
            expected: Vec::new(),
            actual: Vec::new(),
            expected_missing_nl: false,
            actual_missing_nl: false,
        }
    }
}

// Produces a diff between the expected output and actual output.
fn make_diff(expected: &[u8], actual: &[u8]) -> Vec<Mismatch> {
    let mut line_number_expected = 1;
    let mut line_number_actual = 1;
    let mut results = Vec::new();
    let mut mismatch = Mismatch::new(line_number_expected, line_number_actual);

    let mut expected_lines: Vec<&[u8]> = expected.split(|&c| c == b'\n').collect();
    let mut actual_lines: Vec<&[u8]> = actual.split(|&c| c == b'\n').collect();

    debug_assert_eq!(b"".split(|&c| c == b'\n').count(), 1);
    // ^ means that underflow here is impossible
    let expected_lines_count = expected_lines.len() - 1;
    let actual_lines_count = actual_lines.len() - 1;

    if expected_lines.last() == Some(&&b""[..]) {
        expected_lines.pop();
    }

    if actual_lines.last() == Some(&&b""[..]) {
        actual_lines.pop();
    }

    for result in diff::slice(&expected_lines, &actual_lines) {
        match result {
            diff::Result::Left(str) => {
                if !mismatch.actual.is_empty() && !mismatch.actual_missing_nl {
                    results.push(mismatch);
                    mismatch = Mismatch::new(line_number_expected, line_number_actual);
                }
                mismatch.expected.push(str.to_vec());
                mismatch.expected_missing_nl = line_number_expected > expected_lines_count;
                line_number_expected += 1;
            }
            diff::Result::Right(str) => {
                mismatch.actual.push(str.to_vec());
                mismatch.actual_missing_nl = line_number_actual > actual_lines_count;
                line_number_actual += 1;
            }
            diff::Result::Both(str, _) => {
                match (
                    line_number_expected > expected_lines_count,
                    line_number_actual > actual_lines_count,
                ) {
                    (true, false) => {
                        line_number_expected += 1;
                        line_number_actual += 1;
                        mismatch.expected.push(str.to_vec());
                        mismatch.expected_missing_nl = true;
                        mismatch.actual.push(str.to_vec());
                    }
                    (false, true) => {
                        line_number_expected += 1;
                        line_number_actual += 1;
                        mismatch.actual.push(str.to_vec());
                        mismatch.actual_missing_nl = true;
                        mismatch.expected.push(str.to_vec());
                    }
                    (true, true) | (false, false) => {
                        line_number_expected += 1;
                        line_number_actual += 1;
                        if !mismatch.actual.is_empty() || !mismatch.expected.is_empty() {
                            results.push(mismatch);
                            mismatch = Mismatch::new(line_number_expected, line_number_actual);
                        } else {
                            mismatch.line_number_expected = line_number_expected;
                            mismatch.line_number_actual = line_number_actual;
                        }
                    }
                }
            }
        }
    }

    if !mismatch.actual.is_empty() || !mismatch.expected.is_empty() {
        results.push(mismatch);
    }

    results
}

pub fn diff(expected: &[u8], actual: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let diff_results = make_diff(expected, actual);
    for result in diff_results {
        let line_number_expected = result.line_number_expected;
        let line_number_actual = result.line_number_actual;
        let expected_count = result.expected.len();
        let actual_count = result.actual.len();
        match (expected_count, actual_count) {
            (0, 0) => unreachable!(),
            (0, _) => writeln!(
                &mut output,
                "{}a{},{}",
                line_number_expected - 1,
                line_number_actual,
                line_number_actual + actual_count - 1
            )
            .unwrap(),
            (_, 0) => writeln!(
                &mut output,
                "{},{}d{}",
                line_number_expected,
                expected_count + line_number_expected - 1,
                line_number_actual - 1
            )
            .unwrap(),
            _ => writeln!(
                &mut output,
                "{},{}c{},{}",
                line_number_expected,
                expected_count + line_number_expected - 1,
                line_number_actual,
                actual_count + line_number_actual - 1
            )
            .unwrap(),
        }
        for expected in &result.expected {
            write!(&mut output, "< ").unwrap();
            output.write_all(expected).unwrap();
            writeln!(&mut output).unwrap();
        }
        if result.expected_missing_nl {
            writeln!(&mut output, r"\ No newline at end of file").unwrap();
        }
        if expected_count != 0 && actual_count != 0 {
            writeln!(&mut output, "---").unwrap();
        }
        for actual in &result.actual {
            write!(&mut output, "> ").unwrap();
            output.write_all(actual).unwrap();
            writeln!(&mut output).unwrap();
        }
        if result.actual_missing_nl {
            writeln!(&mut output, r"\ No newline at end of file").unwrap();
        }
    }
    output
}

#[test]
fn test_permutations() {
    // test all possible six-line files.
    let _ = std::fs::create_dir("target");
    for &a in &[0, 1, 2] {
        for &b in &[0, 1, 2] {
            for &c in &[0, 1, 2] {
                for &d in &[0, 1, 2] {
                    for &e in &[0, 1, 2] {
                        for &f in &[0, 1, 2] {
                            use std::fs::{self, File};
                            use std::io::Write;
                            use std::process::Command;
                            let mut alef = Vec::new();
                            let mut bet = Vec::new();
                            alef.write_all(if a == 0 { b"a\n" } else { b"b\n" })
                                .unwrap();
                            if a != 2 {
                                bet.write_all(b"b\n").unwrap();
                            }
                            alef.write_all(if b == 0 { b"c\n" } else { b"d\n" })
                                .unwrap();
                            if b != 2 {
                                bet.write_all(b"d\n").unwrap();
                            }
                            alef.write_all(if c == 0 { b"e\n" } else { b"f\n" })
                                .unwrap();
                            if c != 2 {
                                bet.write_all(b"f\n").unwrap();
                            }
                            alef.write_all(if d == 0 { b"g\n" } else { b"h\n" })
                                .unwrap();
                            if d != 2 {
                                bet.write_all(b"h\n").unwrap();
                            }
                            alef.write_all(if e == 0 { b"i\n" } else { b"j\n" })
                                .unwrap();
                            if e != 2 {
                                bet.write_all(b"j\n").unwrap();
                            }
                            alef.write_all(if f == 0 { b"k\n" } else { b"l\n" })
                                .unwrap();
                            if f != 2 {
                                bet.write_all(b"l\n").unwrap();
                            }
                            // This test diff is intentionally reversed.
                            // We want it to turn the alef into bet.
                            let diff = diff(&alef, &bet);
                            File::create("target/ab.diff")
                                .unwrap()
                                .write_all(&diff)
                                .unwrap();
                            let mut fa = File::create("target/alef").unwrap();
                            fa.write_all(&alef[..]).unwrap();
                            let mut fb = File::create("target/bet").unwrap();
                            fb.write_all(&bet[..]).unwrap();
                            let _ = fa;
                            let _ = fb;
                            let output = Command::new("patch")
                                .arg("-p0")
                                .arg("target/alef")
                                .stdin(File::open("target/ab.diff").unwrap())
                                .output()
                                .unwrap();
                            if !output.status.success() {
                                panic!("{:?}", output);
                            }
                            //println!("{}", String::from_utf8_lossy(&output.stdout));
                            //println!("{}", String::from_utf8_lossy(&output.stderr));
                            let alef = fs::read("target/alef").unwrap();
                            assert_eq!(alef, bet);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_permutations_missing_line_ending() {
    // test all possible six-line files with missing newlines.
    let _ = std::fs::create_dir("target");
    for &a in &[0, 1, 2] {
        for &b in &[0, 1, 2] {
            for &c in &[0, 1, 2] {
                for &d in &[0, 1, 2] {
                    for &e in &[0, 1, 2] {
                        for &f in &[0, 1, 2] {
                            for &g in &[0, 1, 2] {
                                use std::fs::{self, File};
                                use std::io::Write;
                                use std::process::Command;
                                let mut alef = Vec::new();
                                let mut bet = Vec::new();
                                alef.write_all(if a == 0 { b"a\n" } else { b"b\n" })
                                    .unwrap();
                                if a != 2 {
                                    bet.write_all(b"b\n").unwrap();
                                }
                                alef.write_all(if b == 0 { b"c\n" } else { b"d\n" })
                                    .unwrap();
                                if b != 2 {
                                    bet.write_all(b"d\n").unwrap();
                                }
                                alef.write_all(if c == 0 { b"e\n" } else { b"f\n" })
                                    .unwrap();
                                if c != 2 {
                                    bet.write_all(b"f\n").unwrap();
                                }
                                alef.write_all(if d == 0 { b"g\n" } else { b"h\n" })
                                    .unwrap();
                                if d != 2 {
                                    bet.write_all(b"h\n").unwrap();
                                }
                                alef.write_all(if e == 0 { b"i\n" } else { b"j\n" })
                                    .unwrap();
                                if e != 2 {
                                    bet.write_all(b"j\n").unwrap();
                                }
                                alef.write_all(if f == 0 { b"k\n" } else { b"l\n" })
                                    .unwrap();
                                if f != 2 {
                                    bet.write_all(b"l\n").unwrap();
                                }
                                match g {
                                    0 => {
                                        alef.pop();
                                    }
                                    1 => {
                                        bet.pop();
                                    }
                                    2 => {
                                        alef.pop();
                                        bet.pop();
                                    }
                                    _ => unreachable!(),
                                }
                                // This test diff is intentionally reversed.
                                // We want it to turn the alef into bet.
                                let diff = diff(&alef, &bet);
                                File::create("target/abn.diff")
                                    .unwrap()
                                    .write_all(&diff)
                                    .unwrap();
                                let mut fa = File::create("target/alefn").unwrap();
                                fa.write_all(&alef[..]).unwrap();
                                let mut fb = File::create("target/betn").unwrap();
                                fb.write_all(&bet[..]).unwrap();
                                let _ = fa;
                                let _ = fb;
                                let output = Command::new("patch")
                                    .arg("-p0")
                                    .arg("--normal")
                                    .arg("target/alefn")
                                    .stdin(File::open("target/abn.diff").unwrap())
                                    .output()
                                    .unwrap();
                                if !output.status.success() {
                                    panic!("{:?}", output);
                                }
                                //println!("{}", String::from_utf8_lossy(&output.stdout));
                                //println!("{}", String::from_utf8_lossy(&output.stderr));
                                let alef = fs::read("target/alefn").unwrap();
                                assert_eq!(alef, bet);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_permutations_empty_lines() {
    // test all possible six-line files with missing newlines.
    let _ = std::fs::create_dir("target");
    for &a in &[0, 1, 2] {
        for &b in &[0, 1, 2] {
            for &c in &[0, 1, 2] {
                for &d in &[0, 1, 2] {
                    for &e in &[0, 1, 2] {
                        for &f in &[0, 1, 2] {
                            use std::fs::{self, File};
                            use std::io::Write;
                            use std::process::Command;
                            let mut alef = Vec::new();
                            let mut bet = Vec::new();
                            alef.write_all(if a == 0 { b"\n" } else { b"b\n" }).unwrap();
                            if a != 2 {
                                bet.write_all(b"b\n").unwrap();
                            }
                            alef.write_all(if b == 0 { b"\n" } else { b"d\n" }).unwrap();
                            if b != 2 {
                                bet.write_all(b"d\n").unwrap();
                            }
                            alef.write_all(if c == 0 { b"\n" } else { b"f\n" }).unwrap();
                            if c != 2 {
                                bet.write_all(b"f\n").unwrap();
                            }
                            alef.write_all(if d == 0 { b"\n" } else { b"h\n" }).unwrap();
                            if d != 2 {
                                bet.write_all(b"h\n").unwrap();
                            }
                            alef.write_all(if e == 0 { b"\n" } else { b"j\n" }).unwrap();
                            if e != 2 {
                                bet.write_all(b"j\n").unwrap();
                            }
                            alef.write_all(if f == 0 { b"\n" } else { b"l\n" }).unwrap();
                            if f != 2 {
                                bet.write_all(b"l\n").unwrap();
                            }
                            // This test diff is intentionally reversed.
                            // We want it to turn the alef into bet.
                            let diff = diff(&alef, &bet);
                            File::create("target/ab_.diff")
                                .unwrap()
                                .write_all(&diff)
                                .unwrap();
                            let mut fa = File::create("target/alef_").unwrap();
                            fa.write_all(&alef[..]).unwrap();
                            let mut fb = File::create("target/bet_").unwrap();
                            fb.write_all(&bet[..]).unwrap();
                            let _ = fa;
                            let _ = fb;
                            let output = Command::new("patch")
                                .arg("-p0")
                                .arg("target/alef_")
                                .stdin(File::open("target/ab_.diff").unwrap())
                                .output()
                                .unwrap();
                            if !output.status.success() {
                                panic!("{:?}", output);
                            }
                            //println!("{}", String::from_utf8_lossy(&output.stdout));
                            //println!("{}", String::from_utf8_lossy(&output.stderr));
                            let alef = fs::read("target/alef_").unwrap();
                            assert_eq!(alef, bet);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_permutations_reverse() {
    // test all possible six-line files.
    let _ = std::fs::create_dir("target");
    for &a in &[0, 1, 2] {
        for &b in &[0, 1, 2] {
            for &c in &[0, 1, 2] {
                for &d in &[0, 1, 2] {
                    for &e in &[0, 1, 2] {
                        for &f in &[0, 1, 2] {
                            use std::fs::{self, File};
                            use std::io::Write;
                            use std::process::Command;
                            let mut alef = Vec::new();
                            let mut bet = Vec::new();
                            alef.write_all(if a == 0 { b"a\n" } else { b"f\n" })
                                .unwrap();
                            if a != 2 {
                                bet.write_all(b"a\n").unwrap();
                            }
                            alef.write_all(if b == 0 { b"b\n" } else { b"e\n" })
                                .unwrap();
                            if b != 2 {
                                bet.write_all(b"b\n").unwrap();
                            }
                            alef.write_all(if c == 0 { b"c\n" } else { b"d\n" })
                                .unwrap();
                            if c != 2 {
                                bet.write_all(b"c\n").unwrap();
                            }
                            alef.write_all(if d == 0 { b"d\n" } else { b"c\n" })
                                .unwrap();
                            if d != 2 {
                                bet.write_all(b"d\n").unwrap();
                            }
                            alef.write_all(if e == 0 { b"e\n" } else { b"b\n" })
                                .unwrap();
                            if e != 2 {
                                bet.write_all(b"e\n").unwrap();
                            }
                            alef.write_all(if f == 0 { b"f\n" } else { b"a\n" })
                                .unwrap();
                            if f != 2 {
                                bet.write_all(b"f\n").unwrap();
                            }
                            // This test diff is intentionally reversed.
                            // We want it to turn the alef into bet.
                            let diff = diff(&alef, &bet);
                            File::create("target/abr.diff")
                                .unwrap()
                                .write_all(&diff)
                                .unwrap();
                            let mut fa = File::create("target/alefr").unwrap();
                            fa.write_all(&alef[..]).unwrap();
                            let mut fb = File::create("target/betr").unwrap();
                            fb.write_all(&bet[..]).unwrap();
                            let _ = fa;
                            let _ = fb;
                            let output = Command::new("patch")
                                .arg("-p0")
                                .arg("target/alefr")
                                .stdin(File::open("target/abr.diff").unwrap())
                                .output()
                                .unwrap();
                            if !output.status.success() {
                                panic!("{:?}", output);
                            }
                            //println!("{}", String::from_utf8_lossy(&output.stdout));
                            //println!("{}", String::from_utf8_lossy(&output.stderr));
                            let alef = fs::read("target/alefr").unwrap();
                            assert_eq!(alef, bet);
                        }
                    }
                }
            }
        }
    }
}