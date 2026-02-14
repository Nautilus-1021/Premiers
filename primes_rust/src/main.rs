use std::{sync::atomic::{AtomicU32, AtomicUsize, Ordering}, thread, time::Instant};

const NMB_PREMIERS: usize = 1_000_000;

fn main() {
    let mut values = [const { AtomicU32::new(0) }; NMB_PREMIERS];
    let cnt = AtomicU32::new(3);
    let prime_cnt = AtomicUsize::new(0);

    println!("Génération des nombres premiers");

    let t1 = Instant::now();
    thread::scope(|th_scope| {
        for _ in 0..4 {
            th_scope.spawn(|| {
                let mut local_pr_cnt = prime_cnt.load(Ordering::SeqCst);
                let mut act_nmb;
                'outer: while local_pr_cnt < NMB_PREMIERS {
                    act_nmb = cnt.fetch_add(2, Ordering::SeqCst);
    
                    local_pr_cnt = prime_cnt.load(Ordering::SeqCst);
                    let sqrt_nmb = (act_nmb as f64).sqrt() as u32 + 20;
                    for pr_idx in 0..local_pr_cnt {
                        let act_pr = values[pr_idx].load(Ordering::SeqCst);
                        if act_pr == 0 {
                            panic!("0 encountered at #{pr_idx}")
                        }
                        /*if act_pr == sqrt_nmb {
                            println!("Carré parfait ! ({act_pr})");
                        }*/
                        if act_pr > sqrt_nmb {
                            break;
                        }

                        if act_nmb % act_pr == 0 {
                            continue 'outer;
                        }
                    }
    
                    local_pr_cnt = prime_cnt.fetch_add(1, Ordering::SeqCst);
                    if local_pr_cnt < NMB_PREMIERS {
                        values[local_pr_cnt].store(act_nmb, Ordering::SeqCst);
                    }
                }
            });
        }
    });

    let tt = t1.elapsed();

    println!("Tri des nombres");

    values.sort_unstable_by(|a , b| a.load(Ordering::SeqCst).cmp(&b.load(Ordering::SeqCst)));

    println!("Vérification");

    let mut pr_cnt = 0;
    let mut cnt = 3;
    while pr_cnt < NMB_PREMIERS {
        let chk_pr = values[pr_cnt].load(Ordering::SeqCst);
        let (verif, divi) = verif_premier(cnt);

        if !verif {
            if cnt == chk_pr {
                println!("{cnt} n'est pas premier (divi: {divi})");
                pr_cnt += 1;
            }

            cnt += 2;
            continue;
        }

        if cnt < chk_pr {
            println!("{cnt} passé");
        } else if cnt > chk_pr {
            println!("{cnt}, {chk_pr}, {pr_cnt}");
            panic!("bug");
        } else if cnt == chk_pr {
            pr_cnt += 1;
        }

        cnt += 2;
    }

    println!("Temps: {:.3}s", tt.as_secs_f64());
}

fn verif_premier(nmb: u32) -> (bool, u32) {
    if nmb % 2 == 0 {
        return (false, 2)
    }

    let sqrt_nmb = f64::from(nmb).sqrt().ceil() as u32;
    for chk in (3..=sqrt_nmb).step_by(2) {
        if nmb % chk == 0 {
            return (false, chk)
        }
    }

    (true, 0)
}