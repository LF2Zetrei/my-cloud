"use client"

import type { Metrics } from "@/lib/types"

const content = {
    cpu: "CPU",
    memory: "Memory",
    uptime: "Uptime",
    gb: "GB",
    of: "of",
}

type Props = { metrics: Metrics }

function toGB(kb: number) {
    return (kb / 1024 / 1024).toFixed(1)
}

function formatUptime(secs: number) {
    const d = Math.floor(secs / 86400)
    const h = Math.floor((secs % 86400) / 3600)
    const m = Math.floor((secs % 3600) / 60)
    if (d > 0) return `${d}d ${h}h ${m}m`
    if (h > 0) return `${h}h ${m}m`
    return `${m}m`
}

function Tile({ label, value, sub }: { label: string; value: string; sub?: string }) {
    return (
        <div className="flex flex-col gap-2 p-4 rounded-xl border border-gray-200 dark:border-white/6 bg-white dark:bg-white/3">
            <span className="text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider">{label}</span>
            <span className="text-2xl font-semibold tracking-tight text-gray-900 dark:text-white">{value}</span>
            {sub && <span className="text-xs text-gray-400 dark:text-gray-500">{sub}</span>}
        </div>
    )
}

export function MetricsCard({ metrics }: Props) {
    const memUsed  = toGB(metrics.memory_used)
    const memTotal = toGB(metrics.memory_total)
    const memPct   = ((metrics.memory_used / metrics.memory_total) * 100).toFixed(0)

    return (
        <div className="grid grid-cols-3 gap-3">
            <Tile
                label={content.cpu}
                value={`${metrics.cpu_usage.toFixed(1)}%`}
            />
            <Tile
                label={content.memory}
                value={`${memUsed} ${content.gb}`}
                sub={`${content.of} ${memTotal} ${content.gb} — ${memPct}%`}
            />
            <Tile
                label={content.uptime}
                value={formatUptime(metrics.uptime_secs)}
            />
        </div>
    )
}
