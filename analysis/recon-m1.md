# M1.0 recon

## Tick cost, native release, by loose units (tuning 1)

| units | ms per tick (mean of 300 ticks) |
|---|---|
| 500 | 0.480 |
| 1000 | 3.404 |
| 2000 | 7.558 |

## Units that fill each cup to its rim, by tuning

Hold spout 1 over a standing cup long enough to overfill it, let go, wait 600 ticks, count the units below the rim.

| cup | tuning | units inside at rest | emitted | wasted |
|---|---|---|---|---|
| regular | 0 | 124 | 228 | 93 |
| regular | 1 | 121 | 224 | 95 |
| regular | 2 | 116 | 221 | 100 |
| small | 0 | 58 | 138 | 77 |
| small | 1 | 56 | 134 | 75 |
| small | 2 | 60 | 131 | 66 |

## A pile of about 60 units poured from one point, after 600 ticks

Poured onto the floor of a cup 240 cm wide. Slope is height over half-width of the base, as a rise per run.

| tuning | units | height cm | half-width cm | slope | max speed at 600, cm/tick |
|---|---|---|---|---|---|
| 0 | 72 | 17.4 | 48.0 | 0.36 | 0.074 |
| 1 | 68 | 20.9 | 55.1 | 0.38 | 0.064 |
| 2 | 65 | 16.6 | 49.3 | 0.34 | 0.057 |
| control: no slope rule | 68 | 7.5 | 120.0 | 0.06 | 0.000 |

## Settling: ticks after the last unit lands until no unit moves faster than 0.05 cm/tick

| tuning | ticks to rest (from a filled cup) |
|---|---|
| 0 | 436 |
| 1 | 172 |
| 2 | 280 |
