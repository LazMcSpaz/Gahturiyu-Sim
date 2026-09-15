#!/usr/bin/env node
/* Scratch harness: node test/probe.js "<expression using the engine>" */
const path = require('path');
const root = path.join(__dirname, '..');
const E = require(path.join(root, 'build.js')).loadEngine(root);
const { newWorld, advance, ageOf } = E;
const sim = (seed, turns, pop = 150, lever = 'none') => {
  let s = newWorld({ seed, population: pop, startYear: 812 });
  const hist = [];
  for (let i = 0; i < turns; i++) { s = advance(s, { lever }); hist.push(s.stats[s.stats.length - 1]); }
  return { s, hist };
};
global.E = E; global.sim = sim; global.newWorld = newWorld; global.advance = advance; global.ageOf = ageOf;
eval(process.argv[2]);
