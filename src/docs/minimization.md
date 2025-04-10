# Minimization

The remaining parts include all of the options and controls retaining to the minimization of the simulation.

## Minimization Target

This section contains the target selection of the minimization.
The target is the value getting changed and minimized by the function with each iteration.
The available choices are:

| mu_max | n_vcd | Feed rate | Glucose | Glutamine | DO | Product |
| ------ | ----- | --------- | ------- | --------- | -- | ------- |


## Minimization mode

You have a choice of selectin the mode of operation for the minimization function value.
Each iteration of the minimization returns a value, that is then read to grade the best minimization.
This is done by selecting the simulation values at each experimental nodes position in time, and returning the sum of the difference between the experimental value and the simulation.
Selecting the specific mode will only look at values of that parameter, while selecting *Mixed* will simply look at each one and returning the sum of all differences.


## Running the minimization

*after* importing the experimental nodes and selecting your wanted target and mode, you can click the ***Minimize*** button and the software will try to minimize the simulation to the experimental data.

<div class="warning">
  For minimization to work best, it is best, to first try to manually aproximate the simulation to the experimental data.
</div>

The simulation parameters will automatically update to reflect the minimized solution.

