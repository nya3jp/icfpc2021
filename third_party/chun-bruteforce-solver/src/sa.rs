// Copyright 2021 Team Special Weekend
// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use rand::prelude::*;
use std::{thread, time::SystemTime};

#[derive(Clone)]
pub struct AnnealingOptions {
    pub time_limit: f64,
    pub limit_temp: f64,
    pub restart: usize,
    pub threads: usize,
    pub silent: bool,
}

pub trait Annealer {
    type State: Clone + Send + Sync;
    type Move;

    fn init_state(&self, rng: &mut impl Rng) -> Self::State;
    fn start_temp(&self, init_score: f64) -> f64;

    fn is_done(&self, _score: f64) -> bool {
        false
    }

    fn eval(&self, state: &Self::State) -> f64;

    fn neighbour(&self, state: &mut Self::State, rng: &mut impl Rng) -> Self::Move;

    fn apply(&self, state: &mut Self::State, mov: &Self::Move);
    fn unapply(&self, state: &mut Self::State, mov: &Self::Move);

    fn apply_and_eval(&self, state: &mut Self::State, mov: &Self::Move, _prev_score: f64) -> f64 {
        self.apply(state, mov);
        self.eval(state)
    }
}

pub fn annealing<A: 'static + Annealer + Clone + Send>(
    annealer: &A,
    opt: &AnnealingOptions,
    seed: u64,
) -> (f64, <A as Annealer>::State) {
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

        ths.into_iter()
            .map(|th| th.join().unwrap())
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap()
    }
}

fn do_annealing<A: Annealer>(
    thread_id: Option<usize>,
    annealer: &A,
    opt: &AnnealingOptions,
    seed: u64,
) -> (f64, <A as Annealer>::State) {
    let mut rng = SmallRng::seed_from_u64(seed);

    let mut state = annealer.init_state(&mut rng);
    let mut cur_score = annealer.eval(&state);
    let mut best_score = cur_score;
    let mut best_ans = state.clone();

    macro_rules! progress {
        ($($arg:expr),*) => {
            if !opt.silent {
                if let Some(tid) = thread_id {
                    eprint!("[{:02}] ", tid);
                }
                eprintln!($($arg),*);
            }
        };
    }

    progress!("Initial score: {}", cur_score);

    let mut restart_cnt = 0;

    let t_max = annealer.start_temp(cur_score);
    let t_min = opt.limit_temp;

    let timer = SystemTime::now();

    let mut temp = t_max;
    let mut progress_ratio = 0.0;
    for i in 0.. {
        if i % 100 == 0 {
            progress_ratio = timer.elapsed().unwrap().as_secs_f64() / opt.time_limit;
            if progress_ratio >= 1.0 {
                break;
            }

            temp = t_max * (t_min / t_max).powf(progress_ratio);
        }

        if temp < t_min {
            restart_cnt += 1;
            if restart_cnt >= opt.restart {
                break;
            }
            progress!("Restarting... {}/{}", restart_cnt, opt.restart);
            temp = t_max;
        }

        let mov = annealer.neighbour(&mut state, &mut rng);
        let new_score = annealer.apply_and_eval(&mut state, &mov, cur_score);

        if new_score <= cur_score
            || rng.gen::<f64>() <= ((cur_score - new_score) as f64 / temp).exp()
        {
            cur_score = new_score;
            if cur_score < best_score {
                if best_score - cur_score > 1e-6 {
                    progress!(
                        "Best: score = {:.3}, temp = {:.9}, progress: {:.2}%",
                        cur_score,
                        temp,
                        progress_ratio * 100.0
                    );
                }
                best_score = cur_score;
                best_ans = state.clone();
            }
            if annealer.is_done(cur_score) {
                break;
            }
        } else {
            annealer.unapply(&mut state, &mov);
        }
    }
    (best_score, best_ans)
}
