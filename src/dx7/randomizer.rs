//! DX7 patch randomizer based on the Synthmata editor
//! (https://synthmata.com/volca-fm/)

// Timbre parameters: atonality, complexity, brightness (all 0...99)
// Envelope parameters: hardness, hit, twang, longness (all 0...99)
// Movement parameters: wobble, wubble, velocity (all 0...99)
// These can all be expressed with sevenate::dx7::Level.

use std::cmp;

use rand::Rng;

use sevenate::Ranged;
use sevenate::dx7::{
    Level,
    Algorithm
};
use sevenate::dx7::voice::Voice;

struct TimbreParameters {
    atonality: Level,
    complexity: Level,
    brightness: Level,
}

struct EnvelopeParameters {
    hardness: Level,
    hitness: Level,
    twang: Level,
    longness: Level,
}

struct MovementParameters {
    wobble: Level,
    wubble: Level,
    velocity: Level,
}

struct RandomizationParameters {
    timbre: TimbreParameters,
    envelope: EnvelopeParameters,
    movement: MovementParameters,
}

fn randomize(params: RandomizationParameters) -> Voice {
    // DX7 algorithms from least complex to most.
    let algorithm_lookup = [
        32, 31, 25, 24, 30, 29, 23, 22, 21, 5, 6, 28, 27, 26, 19, 20,
        1, 2, 4, 3, 9, 11, 10, 12, 13, 8, 7, 15, 14, 17, 16, 18,
    ];

    // Which operators are carriers in a given algorithm.
    // Indexed by algorithm# - 1.
    let carrier_lookup = [
        vec![1, 3],
        vec![1, 3],
        vec![1, 4],
        vec![1, 4],
        vec![1, 3, 5],
        vec![1, 3, 5],
        vec![1, 3],
        vec![1, 3],
        vec![1, 3],
        vec![1, 4],
        vec![1, 4],
        vec![1, 3],
        vec![1, 3],
        vec![1, 3],
        vec![1, 3],
        vec![1],
        vec![1],
        vec![1],
        vec![1, 4, 5],
        vec![1, 2, 4],
        vec![1, 2, 4, 5],
        vec![1, 3, 4, 5],
        vec![1, 2, 4, 5],
        vec![1, 2, 3, 4, 5],
        vec![1, 2, 3, 4, 5],
        vec![1, 2, 4],
        vec![1, 2, 4],
        vec![1, 3, 6],
        vec![1, 2, 3, 5],
        vec![1, 2, 3, 6],
        vec![1, 2, 3, 4, 5],
        vec![1, 2, 3, 4, 5, 6],
    ];

    // Select an algorithm.

    let count = algorithm_lookup.len() as i32;
    let q = count / 8;
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(-q..=q);

    /*
        Math.max(0,
            Math.min(ALGO_COMPLEXITY_LOOKUP.length-1,
                 Math.floor(ALGO_COMPLEXITY_LOOKUP.length/100.0*complexity) + randomInt(-ALGO_COMPLEXITY_LOOKUP.length/8, ALGO_COMPLEXITY_LOOKUP.length/8)))

    ) */
    let chosen_alg = algorithm_lookup[
        cmp::max(
            0,
            cmp::min(
                count - 1,
                (((count as f32) / 100.0 * params.timbre.complexity.value() as f32).floor()) as i32 + x)) as usize];
    let algorithm = Algorithm::new(chosen_alg + 1);
    println!("Algorithm = {}", algorithm.value());

    let mut voice = Voice::new();
    voice.alg = algorithm;

    // Set operator levels. Carriers should be well audible.
    let carriers = &carrier_lookup[(algorithm.value() - 1) as usize];
    for carrier_op in carriers.iter() {
        voice.operators[carrier_op - 1].output_level = Level::new(rng.gen_range(90..=99));
    }

    // ...and so on... (WIP)

    voice
}
