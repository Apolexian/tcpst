#!/bin/bash

~/mpstk/bin/mpstk verify -dks --properties=deadlock-freedom,liveness,liveness+,liveness++,termination,safety pop3.ctx
