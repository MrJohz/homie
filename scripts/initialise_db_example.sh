#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2023 Jonathan Frere
#
# SPDX-License-Identifier: MPL-2.0

##
# Example of a convenience script for setting up a database
# edit this and copy it to e.g. `initialise_db.sh`
#
# Running instructions:
#   ./initialise_db /path/to/target/db
##

set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

BINARY=$1
$BINARY migrate

$BINARY add-user \
    --name="User A" \
    --password="test-password-123"

$BINARY add-user \
    --name="User B" \
    --password="test-password-123"

$BINARY add-task \
      --name "en=Clean the dishes" \
      --name "de=Geschirr spülen" \
      --routine "interval" \
      --duration 14 \
      --participant "User A" \
      --participant "User B" \
      --starts-with "User A" \
      --starts-on 2022-01-01

$BINARY add-task \
      --name "en=Cook dinner" \
      --name "de=Essen kochen" \
      --routine "schedule" \
      --duration 14 \
      --participant "User A" \
      --participant "User B" \
      --starts-with "User A" \
      --starts-on 2022-01-01
