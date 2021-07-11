use rand::prelude::*;
use std::{thread, time::SystemTime};

#[derive(Clone)]
pub struct AnnealingOptions {
    pub time_limit: f64,
    pub limit_temp: f64,
    pub restart: usize,
    pub threads: usize,
    pub silent: bool,
    pub header: String,
}

pub trait Annealer {
    type State: Clone + Send + Sync;
    type Move;

    fn init_state(&self, rng: &mut impl Rng) -> Self::State;
    fn start_temp(&self, init_score: f64) -> f64;

    fn is_done(&self, _score: f64) -> bool {
        false
    }

    fn eval(&self, state: &Self::State, best_score: f64, valid_best_score: f64) -> (f64, bool);

    fn neighbour(
        &self,
        state: &mut Self::State,
        rng: &mut impl Rng,
        progress_ratio: f64,
    ) -> Self::Move;

    fn apply(&self, state: &mut Self::State, mov: &Self::Move);
    fn unapply(&self, state: &mut Self::State, mov: &Self::Move);

    fn apply_and_eval(
        &self,
        state: &mut Self::State,
        mov: &Self::Move,
        best_score: f64,
        valid_best_score: f64,
        _prev_score: f64,
    ) -> (f64, bool) {
        self.apply(state, mov);
        self.eval(state, best_score, valid_best_score)
    }
}

pub fn annealing<A: 'static + Annealer + Clone + Send>(
    annealer: &A,
    opt: &AnnealingOptions,
    seed: u64,
) -> Option<(f64, <A as Annealer>::State)> {
    assert!(opt.threads > 0);

    if opt.threads == 1 {
        do_annealing(None, annealer, opt, seed)
    } else {
        let mut ths = vec![];
        let mut rng = StdRng::seed_from_u64(seed);

        for i in 0..opt.threads {
            let a = annealer.clone();
            let o = opt.clone();
            let tl_seed = rng.gen();
            ths.push(thread::spawn(move || {
                do_annealing(Some(i), &a, &o, tl_seed)
            }));
        }

        let res = ths
            .into_iter()
            .map(|th| th.join().unwrap())
            .collect::<Vec<_>>();

        if !opt.silent {
            eprintln!("===== results =====");
            for (i, r) in res.iter().enumerate() {
                eprintln!(
                    "[{}]: score: {}",
                    i,
                    r.as_ref().map_or(f64::INFINITY, |r| r.0)
                );
            }
        }

        res.into_iter()
            .filter_map(|th| th)
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
    }
}

fn do_annealing<A: Annealer>(
    thread_id: Option<usize>,
    annealer: &A,
    opt: &AnnealingOptions,
    seed: u64,
) -> Option<(f64, <A as Annealer>::State)> {
    let mut rng = SmallRng::seed_from_u64(seed);

    let mut state = annealer.init_state(&mut rng);
    let (mut cur_score, init_state_valid) = annealer.eval(&state, f64::INFINITY, f64::INFINITY);

    let mut best_score = cur_score;

    let mut valid_best_score = f64::INFINITY;
    let mut valid_best_ans = if init_state_valid {
        valid_best_score = cur_score;
        Some(state.clone())
    } else {
        None
    };

    macro_rules! progress {
        ($($arg:expr),*) => {
            if !opt.silent {
                if let Some(tid) = thread_id {
                    eprint!("[{:02}] ", tid);
                }
                eprint!("{}", opt.header);
                eprintln!($($arg),*);
            }
        };
    }

    progress!("Initial score: {}", cur_score);

    let mut restart_cnt = 0;

    let t_max = annealer.start_temp(cur_score);
    let t_min = opt.limit_temp;

    let mut timer = SystemTime::now();
    let time_limit = opt.time_limit;

    let mut temp = t_max;
    let mut progress_ratio = 0.0;
    let mut prev_heart_beat = timer.elapsed().unwrap();

    for i in 0.. {
        if i % 100 == 0 {
            progress_ratio = timer.elapsed().unwrap().as_secs_f64() / time_limit;
            if progress_ratio >= 1.0 {
                restart_cnt += 1;
                if restart_cnt >= opt.restart {
                    progress!(
                        "{} iteration processed, {:.2} iter/s",
                        i,
                        i as f64 / time_limit
                    );
                    break;
                }
                progress!("Restarting... {}/{}", restart_cnt, opt.restart);

                timer = SystemTime::now(); // - Duration::from_secs_f64(time_limit / 2.0);
            }

            temp = t_max * (t_min / t_max).powf(progress_ratio);

            if (timer.elapsed().unwrap() - prev_heart_beat).as_secs_f64() >= 10.0 {
                progress!(
                    "best score = {:12.3}, valid best = {:12.3}, temp = {:12.3}, progress: {:6.2}% 🈚",
                    best_score,
                    valid_best_score,
                    temp,
                    progress_ratio * 100.0
                );
                prev_heart_beat = timer.elapsed().unwrap();
            }
        }

        let mov = annealer.neighbour(&mut state, &mut rng, progress_ratio);

        let (new_score, new_score_valid) =
            annealer.apply_and_eval(&mut state, &mov, best_score, valid_best_score, cur_score);

        if new_score <= cur_score
            || rng.gen::<f64>() <= ((cur_score - new_score) as f64 / temp).exp()
        {
            cur_score = new_score;

            let mut best_updated = false;
            let mut best_valid_updated = false;

            if cur_score < best_score {
                if best_score - cur_score > 1e-6 {
                    best_updated = true;
                }

                best_score = cur_score;
            }

            if new_score_valid && cur_score < valid_best_score {
                if valid_best_score - cur_score > 1e-6 {
                    best_valid_updated = true;
                }

                valid_best_score = cur_score;
                valid_best_ans = Some(state.clone());
            }

            if best_updated || best_valid_updated {
                progress!(
                    "best score = {:12.3}, valid best = {:12.3}, temp = {:12.3}, progress: {:6.2}% {}",
                    best_score,
                    valid_best_score,
                    temp,
                    progress_ratio * 100.0,
                    if best_valid_updated { "✅" } else { "🐴" }
                );
                prev_heart_beat = timer.elapsed().unwrap();
            }

            if annealer.is_done(cur_score) {
                break;
            }
        } else {
            annealer.unapply(&mut state, &mov);
        }
    }

    if valid_best_ans.is_some() {
        Some((valid_best_score, valid_best_ans.unwrap()))
    } else {
        None
    }
}
