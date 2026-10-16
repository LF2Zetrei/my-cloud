"use client"

import Link from "next/link"
import type { Container } from "@/lib/types"

const content = {
    details: "Details",
    delete: "Delete",
    pid: "pid",
}

const statusStyles: Record<string, string> = {
    Running: "bg-emerald-100 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400",
    Stopped: "bg-gray-100 text-gray-500 dark:bg-white/6 dark:text-gray-400",
    Crashed: "bg-red-100 text-red-600 dark:bg-red-500/10 dark:text-red-400",
}

const statusDot: Record<string, string> = {
    Running: "bg-emerald-400",
    Stopped: "bg-gray-400 dark:bg-gray-600",
    Crashed: "bg-red-400",
}

type Props = {
    container: Container
    onDelete: (id: string) => void
}

export function ContainerCard({ container, onDelete }: Props) {
    return (
        <div className="group flex items-center justify-between px-4 py-3 rounded-xl border border-gray-200 dark:border-white/6 bg-white dark:bg-white/3 hover:border-gray-300 dark:hover:border-white/10 transition-colors">
            <div className="flex items-center gap-3 min-w-0">
                <span className={`shrink-0 w-1.5 h-1.5 rounded-full ${statusDot[container.status]}`} />
                <div className="min-w-0">
                    <p className="text-sm font-medium text-gray-900 dark:text-white truncate">{container.name}</p>
                    <p className="text-xs text-gray-400 dark:text-gray-500 font-mono truncate">{container.image}</p>
                </div>
            </div>

            <div className="flex items-center gap-2 shrink-0 ml-3">
                <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${statusStyles[container.status]}`}>
                    {container.status}
                </span>
                {container.status === "Running" && container.pid && (
                    <span className="text-xs text-gray-400 dark:text-gray-500 font-mono hidden group-hover:inline">
                        {content.pid} {container.pid}
                    </span>
                )}
                <Link
                    href={`/containers/${container.id}`}
                    className="text-xs px-2.5 py-1 rounded-lg border border-gray-200 dark:border-white/8 text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white hover:border-gray-300 dark:hover:border-white/20 transition-colors"
                >
                    {content.details}
                </Link>
                <button
                    onClick={() => onDelete(container.id)}
                    className="text-xs px-2.5 py-1 rounded-lg border border-transparent text-gray-400 dark:text-gray-500 hover:border-red-200 dark:hover:border-red-500/30 hover:text-red-500 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-500/8 transition-colors"
                >
                    {content.delete}
                </button>
            </div>
        </div>
    )
}
