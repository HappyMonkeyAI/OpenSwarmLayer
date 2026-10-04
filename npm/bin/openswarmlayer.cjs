#!/usr/bin/env node
'use strict';
const { main } = require('../lib/install.cjs');
main(process.argv.slice(2)).catch((error) => {
  console.error(`OpenSwarmLayer: ${error.message}`);
  process.exitCode = 1;
});
