"use client"

import Link from "next/link"
import { MetricsCard } from "@/components/MetricsCard"
import { ContainerCard } from "@/components/ContainerCard"
import { MetaballHero } from "@/components/MataballHero"
import { useMetrics } from "@/hooks/useMetrics"
import { useContainers } from "@/hooks/useContainer"
import { useStorage } from "@/hooks/useStorage"

const content = {
    metrics: "System metrics",
    noMetrics: "No metrics available",
    recentContainers: "Recent containers",
    seeAllContainers: "See all →",
    noContainers: "No containers yet.",
    running: "running",
    stopped: "stopped",
    crashed: "crashed",
    recentFiles: "Recent files",
    seeAllFiles: "See all →",
    noFiles: "No files yet.",
    containers: "containers",
}

function fileIcon(name: string) {
    const ext = name.split(".").pop()?.toLowerCase() ?? ""
    if (["jpg","jpeg","png","gif","svg","webp"].includes(ext)) return "🖼"
    if (["mp4","mov","avi","mkv"].includes(ext))               return "🎬"
    if (["mp3","wav","ogg"].includes(ext))                     return "🎵"
    if (["zip","tar","gz","rar"].includes(ext))                return "📦"
    if (["json","yaml","yml","toml"].includes(ext))            return "⚙"
    if (["sh","bash","zsh"].includes(ext))                     return "💻"
    return "📄"
}

export default function Home() {
    const { metrics, error: metricsError } = useMetrics()
    const { containers, remove }           = useContainers()
    const { files }                        = useStorage()

    const running = containers.filter((c) => c.status === "Running").length
    const stopped = containers.filter((c) => c.status === "Stopped").length
    const crashed = containers.filter((c) => c.status === "Crashed").length

    return (
        <div className="w-full">
            <MetaballHero subtitle={`${containers.length} ${content.containers} · ${running} ${content.running}`} />

            <main className="p-6 max-w-5xl mx-auto flex flex-col gap-8">

                {/* Metrics */}
                <section>
                    <h2 className="text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-3">
                        {content.metrics}
                    </h2>
                    {metrics
                        ? <MetricsCard metrics={metrics} />
                        : <p className="text-sm text-red-400">{metricsError ?? content.noMetrics}</p>
                    }
                </section>

                {/* Containers + Storage */}
                <div className="grid grid-cols-[1fr_220px] gap-6 items-start">

                    {/* Containers */}
                    <section>
                        <div className="flex items-center justify-between mb-3">
                            <h2 className="text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider">
                                {content.recentContainers}
                            </h2>
                            <Link href="/containers" className="text-xs text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-200 transition-colors">
                                {content.seeAllContainers}
                            </Link>
                        </div>

                        <div className="flex gap-2 mb-3">
                            <span className="text-xs px-2.5 py-1 rounded-full bg-emerald-100 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400">
                                {running} {content.running}
                            </span>
                            <span className="text-xs px-2.5 py-1 rounded-full bg-gray-100 text-gray-500 dark:bg-white/6 dark:text-gray-400">
                                {stopped} {content.stopped}
                            </span>
                            {crashed > 0 && (
                                <span className="text-xs px-2.5 py-1 rounded-full bg-red-100 text-red-600 dark:bg-red-500/10 dark:text-red-400">
                                    {crashed} {content.crashed}
                                </span>
                            )}
                        </div>

                        <div className="flex flex-col gap-2">
                            {containers.length === 0
                                ? <p className="text-sm text-gray-400 dark:text-gray-500">{content.noContainers}</p>
                                : containers.slice(0, 4).map((c) => (
                                    <ContainerCard key={c.id} container={c} onDelete={remove} />
                                ))
                            }
                        </div>
                    </section>

                    {/* Storage */}
                    <section>
                        <div className="flex items-center justify-between mb-3">
                            <h2 className="text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider">
                                {content.recentFiles}
                            </h2>
                            <Link href="/storage" className="text-xs text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-200 transition-colors">
                                {content.seeAllFiles}
                            </Link>
                        </div>

                        <div className="flex flex-col gap-2">
                            {files.length === 0
                                ? <p className="text-sm text-gray-400 dark:text-gray-500">{content.noFiles}</p>
                                : files.slice(0, 6).map((file) => (
                                    <div
                                        key={file}
                                        className="flex items-center gap-2.5 px-3 py-2.5 rounded-xl border border-gray-200 dark:border-white/6 bg-white dark:bg-white/3 hover:border-gray-300 dark:hover:border-white/12 transition-colors"
                                    >
                                        <span className="text-base leading-none">{fileIcon(file)}</span>
                                        <span className="text-xs text-gray-600 dark:text-gray-300 truncate">{file}</span>
                                    </div>
                                ))
                            }
                        </div>
                    </section>

                </div>
            </main>
        </div>
    )
}
