SM2
===

> SuperMemo2 algorithm, written in Rust

Large part of the code in this repo was ported from [Anki][1], which
implements [SM2][2] algorithm.


Copyright: Ankitects Pty Ltd and contributors

License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html


## Installation

```shell
# NPM
npm install @repeatnotes/sm2

# Yarn
yarn add @repeatnotes/sm2
```

## Usage

```javascript
const main = async () => {
    const wasm = await import('@repeatnotes/sm2')

    const sm2 = new wasm.Sm2({
      learn_steps: [1.0, 10.0],
      relearn_steps: [10.0],
      initial_ease: 2500,
      easy_multiplier: 1.3,
      hard_multiplier: 1.2,
      lapse_multiplier: 0.0,
      interval_multiplier: 1.0,
      maximum_review_interval: 36500,
      minimum_review_interval: 1,
      graduating_interval_good: 1,
      graduating_interval_easy: 4,
      leech_threshold: 8,
    })

    const card = {
      card_type: 0,
      card_queue: 0,
      due: 0,
      interval: 0,
      ease_factor: 0,
      reps: 0,
      lapses: 0,
      remaining_steps: 0,
    }

    console.log("Next due with Ok answer:", sm2.next_interval(card, 3))
    console.log("Answer Ok:", sm2.answer_card(card, 3))
}
```

See [lib.rs](src/lib.rs) for full API.


[1]: https://github.com/ankitects/anki
[2]: https://www.supermemo.com/en/archives1990-2015/english/ol/sm2
