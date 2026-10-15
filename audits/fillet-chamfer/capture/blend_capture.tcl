# Capture in the native command's caller scope and preserve its return options.
set audit_blend_index 0
file mkdir capture
foreach {command input} {blend audit_blend_input chamf audit_chamfer_input} {
    if {[llength [info commands $command]] == 0} {continue}
    rename $command audit_original_$command
    proc $command {args} [string map [list @input@ $input @command@ $command] {
    incr ::audit_blend_index
    set prefix [file join [pwd] capture [format {%03d} $::audit_blend_index]]
    set log [open "$prefix.capture.log" w]
    set captured [expr {![catch {uplevel 1 [linsert [lrange $args 1 end] 0 @input@ $prefix]} message]}]
    puts $log [list input_captured $captured message $message]
    set code [catch {uplevel 1 [linsert $args 0 audit_original_@command@]} value options]
    if {$captured && $code == 0} {
        set result_captured [expr {![catch {uplevel 1 [list audit_blend_output $prefix [lindex $args 0]]} message]}]
        puts $log [list output_captured $result_captured message $message]
    }
    puts $log [list native_return_code $code]
    close $log
    return -options $options $value
    }]
}

# Observe successful native configuration calls in the same caller scope.
foreach command {tolblend continuityblend} {
    if {[llength [info commands $command]] == 0} {continue}
    rename $command audit_original_$command
    proc $command {args} [string map [list @command@ $command] {
        set code [catch {uplevel 1 [linsert $args 0 audit_original_@command@]} value options]
        if {$code == 0} {
            catch {uplevel 1 [linsert $args 0 audit_blend_settings @command@]}
        }
        return -options $options $value
    }]
}
