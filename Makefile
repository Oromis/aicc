# Top-level makefile for AICC
# Supported commands:
#   make <sub-project> 							- Builds the given sub-project
#   make <sub-project> [build|flash|run|all]    - Runs the given target of the sub-project
#   make build									- Builds all sub-projects
#   make flash									- Builds and flashes all sub-projects
#   make run									- Runs all sub-projects on the target board
#   make all 									- Builds, flashes and runs all sub-projects

# List of all supported sub-projects
sub_projects = drive-core

# Filters all sub-projects out of the argument list. If the resulting list is not equal to the original list,
# then we want to run a sub-project command and this variable contains all commands to send to this sub-project
sub_targets = $(filter-out $(sub_projects),$(MAKECMDGOALS))

ifeq "$(sub_targets)" "$(MAKECMDGOALS)"
# No sub-project call => provide global "build", "flash", "run" and "all" targets which will run on all targets
# the @echo > /dev/null trick stops make from complaining that the rule contains no commands.
sub_targets = $(MAKECMDGOALS)

all: $(sub_projects)
	@echo > /dev/null
build: $(sub_projects)
	@echo > /dev/null
flash: $(sub_projects)
	@echo > /dev/null
run: $(sub_projects)
	@echo > /dev/null

else
# Sub-project call

# This creates fake targets for all targets to be run on the sub-project
# (to silence make reporting error for missing targets)
$(sub_targets):
	@echo > /dev/null
endif

# This creates build targets for all sub-projects
$(sub_projects):
	@$(MAKE) -C $@ $(sub_targets)

.PHONY: $(sub_projects) $(sub_targets)
