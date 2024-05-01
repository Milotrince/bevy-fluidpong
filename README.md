# FluidPong
FluidPong is a classic two-player game with a twist. In addition to moving the paddles, players can also manipulate the fluid on the board. We offer two different kinds of fluids, 1) SPH (particle-based) rendered with a metaball shader, and 2) Navier-Stokes (grid-based) rendered using bilinear filter shader.

## Running the game
`cargo run -- --fluid sph`
`cargo run -- --fluid ns`

optionally, add debug

`cargo run -- --fluid ns --debug`

### CS 184 

- [Project Proposal](https://cal-cs184-student.github.io/hw-webpages-sp24-oliver-ni/proj/)
- [Milestone](https://cal-cs184-student.github.io/hw-webpages-sp24-oliver-trinity/milestone)
- [Final Deliverable](https://cal-cs184-student.github.io/hw-webpages-sp24-oliver-trinity/final)