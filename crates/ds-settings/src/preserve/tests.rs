use super::merge;
use crate::lenient::UnknownKey;

struct Case {
    name: &'static str,
    existing: &'static str,
    fresh: &'static str,
    unknown: &'static [&'static str],
    expect: &'static str,
}

fn run(case: &Case) -> String {
    let unknown: Vec<UnknownKey> = case
        .unknown
        .iter()
        .map(|key| UnknownKey((*key).to_owned()))
        .collect();
    merge(case.existing, case.fresh, &unknown).unwrap_or_else(|e| panic!("{}: {e}", case.name))
}

const CASES: &[Case] = &[
    Case {
        name: "a changed value keeps its spacing and its comment",
        existing: "# my settings\nsize   =  3 # pixels\nname = \"ada\"\n",
        fresh: "size = 4\nname = \"ada\"\n",
        unknown: &[],
        expect: "# my settings\nsize   =  4 # pixels\nname = \"ada\"\n",
    },
    Case {
        name: "a value that means the same is left as the person wrote it",
        existing: "size = 0x10\nratio = 1.50\n",
        fresh: "size = 16\nratio = 1.5\n",
        unknown: &[],
        expect: "size = 0x10\nratio = 1.50\n",
    },
    Case {
        name: "an unknown key and an unknown table stay",
        existing: "future = \"kept\"\nsize = 3\n[dock]\nsize = 1\npuppy = true\n[later]\nanswer = 42\n",
        fresh: "size = 3\n[dock]\nsize = 2\n",
        unknown: &["future", "dock.puppy", "later"],
        expect: "future = \"kept\"\nsize = 3\n[dock]\nsize = 2\npuppy = true\n[later]\nanswer = 42\n",
    },
    Case {
        name: "a key the type owns and no longer writes is taken out",
        existing: "size = 3\nname = \"ada\"\n[scores]\na = 1\nb = 2\n",
        fresh: "size = 3\n[scores]\na = 1\n",
        unknown: &[],
        expect: "size = 3\n[scores]\na = 1\n",
    },
    Case {
        name: "a table the type owns and no longer writes goes, unless something unknown is in it",
        existing: "[gone]\nx = 1\n[mixed]\nx = 1\nkeep = 2\n",
        fresh: "",
        unknown: &["mixed.keep"],
        expect: "[mixed]\nkeep = 2\n",
    },
    Case {
        name: "a new key and a new table are added",
        existing: "# top\nsize = 3\n",
        fresh: "size = 3\nname = \"ada\"\n[dock]\nsize = 2\n",
        unknown: &[],
        expect: "# top\nsize = 3\nname = \"ada\"\n[dock]\nsize = 2\n",
    },
    Case {
        name: "an inline table is opened so its unknown keys stay",
        existing: "dock = { size = 1, puppy = true }\n",
        fresh: "[dock]\nsize = 2\n",
        unknown: &["dock.puppy"],
        expect: "[dock]\nsize = 2\npuppy = true\n",
    },
    Case {
        name: "an empty file takes the typed document whole",
        existing: "",
        fresh: "size = 3\n[dock]\nsize = 2\n",
        unknown: &[],
        expect: "size = 3\n[dock]\nsize = 2\n",
    },
];

/// The lines of `text` that say something: blank lines are the encoder's business, not the test's.
fn said(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .collect()
}

#[test]
fn the_typed_document_is_laid_over_the_file() {
    for case in CASES {
        assert_eq!(said(&run(case)), said(case.expect), "{}", case.name);
    }
}

#[test]
fn text_that_is_not_toml_is_refused() {
    assert!(merge("size = = 3", "size = 3\n", &[]).is_err());
}
