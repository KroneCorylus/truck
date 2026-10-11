# Export physical selections without changing the upstream blend or its return status.
set audit_blend_index 0
file mkdir capture
rename blend audit_original_blend
proc blend {args} {
    incr ::audit_blend_index
    set prefix [file join [pwd] capture [format {%03d} $::audit_blend_index]]
    set log [open "$prefix.capture.log" w]
    set captured [expr {![catch {uplevel 1 [linsert [lrange $args 1 end] 0 audit_blend_input $prefix]} message]}]
    puts $log [list input_captured $captured message $message]
    set code [catch {uplevel 1 [linsert $args 0 audit_original_blend]} value options]
    if {$captured && $code == 0} {
        set result_captured [expr {![catch {uplevel 1 [list audit_blend_output $prefix [lindex $args 0]]} message]}]
        puts $log [list output_captured $result_captured message $message]
    }
    puts $log [list native_return_code $code]
    close $log
    return -options $options $value
}
