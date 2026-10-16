"use client"

import Link from "next/link"
import { useParams } from "next/navigation"
import { useContainerMetrics } from "@/hooks/useContainerMetrics"
import { useContainers } from "@/hooks/useContainer"

const content = {
    loading: "Loading...",
    notFound: "Container not found.",
    back: "← Back",
    info: {
        title: "Info",
        image: "Image",
        args: "Args",
        noArgs: "—",
        pid: "PID",
        id: "ID",
    },
    metrics: {
        title: "Metrics",
        cpu: "CPU",
        memory: "Memory",
        pid: "PID",
        memoryUnit: "MB",
        unavailable: "Metrics unavailable — container is not running.",
        loading: "Loading metrics...",
    },
}

const statusStyles: Record<string, string> = {
    Running: "bg-emerald-100 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400",
    Stopped: "bg-gray-100 text-gray-500 dark:bg-white/6 dark:text-gray-400",
    Crashed: "bg-red-100 text-red-600 dark:bg-red-500/10 dark:text-red-400",
}

function Row({ label, value }: { label: string; value: React.ReactNode }) {
    return (
        <div className="flex items-start gap-6 py-3 border-b border-gray-100 dark:border-white/4 last:border-0">
            <span className="w-20 shrink-0 text-xs font-medium text-gray-400 dark:text-gray-500 pt-px uppercase tracking-wide">{label}</span>
            <span className="text-sm text-gray-800 dark:text-gray-200 break-all">{value}</span>
        </div>
    )
}

function Tile({ label, value }: { label: string; value: string }) {
    return (
        <div className="flex flex-col gap-2 p-4 rounded-xl border border-gray-200 dark:border-white/6 bg-white dark:bg-white/3">
            <span className="text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider">{label}</span>
            <span className="text-2xl font-semibold tracking-tight text-gray-900 dark:text-white">{value}</span>
        </div>
    )
}

export default function ContainerDetailPage() {
    const { id } = useParams<{ id: string }>()
    const { containers, loading, error } = useContainers()
    const { containerMetrics, error: metricsError } = useContainerMetrics(id)

    const container = containers.find((c) => c.id === id)

    if (loading)    return <p className="p-6 text-sm text-gray-400 dark:text-gray-500">{content.loading}</p>
    if (error)      return <p className="p-6 text-sm text-red-500">{error}</p>
    if (!container) return <p className="p-6 text-sm text-gray-400 dark:text-gray-500">{content.notFound}</p>

    return (
        <div className="p-6 max-w-5xl mx-auto flex flex-col gap-6">

            <Link
                href="/containers"
                className="self-start flex items-center gap-1 text-sm text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
            >
                {content.back}
            </Link>

            <div className="flex items-center gap-3">
                <h1 className="text-lg font-semibold text-gray-900 dark:text-white">{container.name}</h1>
                <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${statusStyles[container.status]}`}>
                    {container.status}
                </span>
            </div>

            {/* Info */}
            <section>
                <h2 className="text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-3">{content.info.title}</h2>
                <div className="rounded-xl border border-gray-200 dark:border-white/6 bg-white dark:bg-white/3 px-4">
                    <Row label={content.info.image} value={<code className="font-mono text-xs">{container.image}</code>} />
                    <Row
                        label={content.info.args}
                        value={
                            container.args.length > 0
                                ? <code className="font-mono text-xs bg-gray-100 dark:bg-white/6 px-1.5 py-0.5 rounded">{container.args.join(" ")}</code>
                                : <span className="text-gray-400 dark:text-gray-600">{content.info.noArgs}</span>
                        }
                    />
                    {container.pid && (
                        <Row label={content.info.pid} value={<code className="font-mono text-xs">{container.pid}</code>} />
                    )}
                    <Row label={content.info.id} value={<code className="font-mono text-xs text-gray-400 dark:text-gray-500">{container.id}</code>} />
                </div>
            </section>

            {/* Metrics */}
            <section>
                <h2 className="text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-3">{content.metrics.title}</h2>
                {container.status !== "Running" ? (
                    <p className="text-sm text-gray-400 dark:text-gray-500">{content.metrics.unavailable}</p>
                ) : metricsError ? (
                    <p className="text-sm text-red-500">{metricsError}</p>
                ) : containerMetrics ? (
                    <div className="grid grid-cols-3 gap-3">
                        <Tile label={content.metrics.cpu}    value={`${containerMetrics.cpu_percent.toFixed(1)}%`} />
                        <Tile label={content.metrics.memory} value={`${(containerMetrics.memory_rss_kb / 1024).toFixed(1)} ${content.metrics.memoryUnit}`} />
                        <Tile label={content.metrics.pid}    value={String(containerMetrics.pid)} />
                    </div>
                ) : (
                    <p className="text-sm text-gray-400 dark:text-gray-500">{content.metrics.loading}</p>
                )}
            </section>
        </div>
    )
}
