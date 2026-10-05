# M1.0 recon

## Tick cost, native release, by loose units (tuning 1)

| units | ms per tick (mean of 300 ticks) |
|---|---|
| 500 | 0.181 |
| 1000 | 0.516 |
| 2000 | 1.435 |

## Units that fill each cup to its rim, by tuning

Hold spout 1 over a standing cup long enough to overfill it, let go, wait 600 ticks, count the units below the rim.

| cup | tuning | units inside at rest | emitted | wasted |
|---|---|---|---|---|
| regular | 0 | 118 | 228 | 81 |
| regular | 1 | 115 | 224 | 83 |
| regular | 2 | 118 | 221 | 80 |
| small | 0 | 60 | 138 | 65 |
| small | 1 | 59 | 134 | 63 |
| small | 2 | 58 | 131 | 56 |

## A pile of about 60 units poured from one point, after 600 ticks

Poured onto the floor of a cup 240 cm wide. Slope is height over half-width of the base, as a rise per run.

| tuning | units | height cm | half-width cm | slope | max speed at 600, cm/tick |
|---|---|---|---|---|---|
| 0 | 72 | 18.2 | 53.7 | 0.34 | 0.000 |
| 1 | 68 | 18.6 | 45.7 | 0.41 | 0.000 |
| 2 | 65 | 18.2 | 52.4 | 0.35 | 0.000 |
| control: no slope rule | 68 | 7.5 | 120.0 | 0.06 | 0.000 |

## Settling: ticks after the last unit lands until no unit moves faster than 0.05 cm/tick

| tuning | ticks to rest (from a filled cup) |
|---|---|
| 0 | 187 |
| 1 | 176 |
| 2 | 108 |
