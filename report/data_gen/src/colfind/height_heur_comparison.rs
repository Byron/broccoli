use crate::inner_prelude::*;

pub fn handle_bench_inner(grow: f64, bot_inner: &mut [isize], height: usize) -> f64 {
    let mut bots = distribute(grow, bot_inner, |a| a.to_f32n());

    bench_closure(|| {
        let mut tree = TreeBuilder::new(&mut bots).with_height(height).build_seq();
        assert_eq!(tree.get_height(), height);

        tree.find_colliding_pairs_mut(|a, b| {
            **a.unpack_inner() += 2;
            **b.unpack_inner() += 2;
        });
    })
}

pub fn handle_theory_inner(grow: f64, bot_inner: &mut [isize], height: usize) -> usize {
    datanum::datanum_test(|maker| {
        let mut bots = distribute(grow, bot_inner, |a| a.to_isize_dnum(maker));

        let mut tree = TreeBuilder::new(&mut bots).with_height(height).build_seq();
        assert_eq!(tree.get_height(), height);

        tree.find_colliding_pairs_mut(|a, b| {
            **a.unpack_inner() += 2;
            **b.unpack_inner() += 2;
        });
    })
}

pub fn handle(fb: &mut FigureBuilder) {
    handle2d(fb);
    handle_lowest(fb);
}

fn handle_lowest(fb: &mut FigureBuilder) {
    struct BenchRecord {
        height: usize,
        num_bots: usize,
    }

    let mut benches: Vec<BenchRecord> = Vec::new();

    let its = (1usize..50_000).step_by(2000);
    for num_bots in its.clone() {
        let mut minimum = None;
        let max_height = (num_bots as f64).log2() as usize;

        let grow = 0.2;
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        for height in 1..max_height {
            let bench = handle_bench_inner(grow, &mut bot_inner, height);
            match minimum {
                Some((a, _b)) => {
                    if bench < a {
                        minimum = Some((bench, height));
                    }
                }
                None => {
                    minimum = Some((bench, height));
                }
            }
        }

        if let Some((_, height)) = minimum {
            benches.push(BenchRecord { height, num_bots });
        }
    }

    {
        let mut fg = fb.build("height_heuristic_vs_optimal");

        let x = benches.iter().map(|a| a.num_bots);
        let y = benches.iter().map(|a| a.height);

        let heur = {
            let mut vec = Vec::new();
            for num_bots in its.clone() {
                let height = TreePreBuilder::new(num_bots).get_height();
                vec.push((num_bots, height));
            }
            vec
        };

        let heurx = heur.iter().map(|a| a.0);
        let heury = heur.iter().map(|a| a.1);
        fg.axes2d()
            .set_legend(
                Graph(1.0),
                Graph(0.0),
                &[LegendOption::Placement(AlignRight, AlignBottom)],
                &[],
            )
            .set_title(
                "Broccoli Colfind Query: Optimal Height vs Heuristic Height with abspiral(x,0.2)",
                &[],
            )
            .set_x_label("Num bots", &[])
            .set_y_label("Best Tree Height", &[])
            .points(
                x,
                y,
                &[
                    Caption("Optimal"),
                    PointSymbol('O'),
                    Color(COLS[0]),
                    PointSize(1.0),
                ],
            )
            .points(
                heurx,
                heury,
                &[
                    Caption("Heuristic"),
                    PointSymbol('x'),
                    Color(COLS[1]),
                    PointSize(2.0),
                ],
            );

        fb.finish(fg);
    }
}

fn handle2d(fb: &mut FigureBuilder) {
    #[derive(Debug)]
    struct Record {
        height: usize,
        num_comparison: usize,
    }

    #[derive(Debug)]
    struct BenchRecord {
        height: usize,
        bench: f64,
    }

    let mut theory_records = Vec::new();

    let mut bench_records: Vec<BenchRecord> = Vec::new();
    let num_bots = 10000;
    let grow = 0.2;
    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

    for height in 2..13 {
        let num_comparison = handle_theory_inner(grow, &mut bot_inner, height);
        theory_records.push(Record {
            height,
            num_comparison,
        });
    }

    for height in (2..13).flat_map(|a| std::iter::repeat(a).take(20)) {
        let bench = handle_bench_inner(grow, &mut bot_inner, height);
        bench_records.push(BenchRecord { height, bench });
    }

    use gnuplot::*;

    let mut fg = fb.build("height_heuristic");

    let x = theory_records.iter().map(|a| a.height);
    let y = theory_records.iter().map(|a| a.num_comparison);

    fg.axes2d()
        .set_pos_grid(2, 1, 0)
        .set_title("Number of Comparisons with different numbers of objects per node with abspiral(10000,0.2)", &[])
        .lines(x,y,&[Color(COLS[0]), LineWidth(2.0)])
        .set_x_label("Tree Height", &[])
        .set_y_label("Number of Comparisons", &[]);

    let x = bench_records.iter().map(|a| a.height);
    let y = bench_records.iter().map(|a| a.bench);

    fg.axes2d()
        .set_pos_grid(2, 1, 1)
        .set_title("Bench times with different numbers of objects per node (seq,colfind) with abspiral(10000,0.2)", &[])
        .points(x,y,&[Color(COLS[0]), LineWidth(2.0)])
        .set_x_label("Tree Height", &[])
        .set_y_label("Time in seconds", &[]);

    fb.finish(fg);
}
