# Data tab

This tab contains three buttons.

## Load Data

Loads a `csv` with data points into the simulation.
The csv must include the following columns:

  | minutes | vcd | gln | gluc | do\_50 | product |
  | ------- | --- | --- | ---- | ------ | ------- |
  | the ammount of time passed in minutes | Viable cell density | glutamine concentration | glucose concentration | dissolved oxygen | product concentration |
  
## Clear Nodes

This button clears the experimental data nodes from the graph.

## Export data

You can use this button to export simulation data.
The program saves a `csv` file with the data nodes of each of the parameters, with step size of 1 minute.
It also saves a `json` file with the same name containing all of the simulation parameters.
This file can then be used with [Load Simulation](control-panel.md#load-simulation) to recreate the simulation state.



