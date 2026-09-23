//! W0 spike: what Blitz @ e99fbdbd actually supports. `cargo run -- all` (or `-- s3 s7`)
//! renders every probe to spike/out/*.png and prints a table. Throwaway code.

mod harness;
mod probes;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let wanted = |id: &str| {
        args.iter()
            .any(|a| a == "all" || a.eq_ignore_ascii_case(id))
    };
    if args.is_empty() {
        eprintln!("usage: blitz-probe all | s1 s2 ...");
        std::process::exit(2);
    }
    let rows: Vec<probes::Row> = probes::ALL
        .iter()
        .filter(|(id, _)| wanted(id))
        .map(|(id, run)| {
            eprintln!("running {id}");
            run()
        })
        .collect();
    println!("| id | result | png | what is visible |");
    println!("|----|--------|-----|-----------------|");
    for r in rows {
        println!(
            "| {} | {} | {} | {} |",
            r.id,
            r.verdict,
            r.pngs.join("<br>"),
            r.seen
        );
    }
}
