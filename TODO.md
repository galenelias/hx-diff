# Little Rocks
- Persist scroll position per file
- Refresh of Workspace. Cmd+R, and upon stage/unstage

# Big Rocks
1. **Incremental** Refresh of Workspace. Cmd+R, and upon stage/unstage
2. Tree structure in file list
3. Reload diff on file change (hard, need watcher and some sort of async hookup)

Done:
- Reset scroll position on file change
- Diff_pane: Scroll to first change
- Line Numbers
 - min_width_for_number_on_gutter
- F8 (jump to next change)
- Scrollbar with diff minimap
- Dragging scroll bar actually scrolls


# Data model plan

- Core 'Worktree' structure, which contains full state of the operation and results.

- TODO: Respond to changes...
- Refresh diff_list
- Create events for (select, revert, etc.)

# Custom Rendering Notes

Element methods:

- request_layout
  Maybe easy? Full width and height.

- prepaint
  What is this? Generate a layout?

- paint
  Seems reasonable. Use layout, and actually draw stuff. Not sure why this is separate from prepaint yet
