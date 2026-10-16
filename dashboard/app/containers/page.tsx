"use client"

import { useState } from "react"
import { ContainerCard } from "@/components/ContainerCard"
import { useContainers } from "@/hooks/useContainer"

const content = {
    title: "Containers",
    total: "total",
    newContainer: "+ New container",
    cancel: "Cancel",
    form: {
        name: "Name",
        namePlaceholder: "my-container",
        image: "Image",
        imagePlaceholder: "/bin/sleep",
        args: "Args",
        argsHint: "(space-separated)",
        argsPlaceholder: "3600",
        startImmediately: "Start immediately",
        create: "Create",
    },
    loading: "Loading...",
    noContainers: "No containers found.",
}

const filters = ["All", "Running", "Stopped", "Crashed"] as const
type Filter = typeof filters[number]

export default function ContainersPage() {
    const { containers, loading, error, create, remove } = useContainers()
    const [filter, setFilter] = useState<Filter>("All")
    const [showForm, setShowForm] = useState(false)
    const [name, setName]   = useState("")
    const [image, setImage] = useState("")
    const [args, setArgs]   = useState("")
    const [start, setStart] = useState(false)

    const handleCreate = async () => {
        const argsArr = args.trim() ? args.trim().split(/\s+/) : []
        await create(name, image, argsArr, start)
        setName(""); setImage(""); setArgs(""); setStart(false)
        setShowForm(false)
    }

    const filtered = filter === "All"
        ? containers
        : containers.filter((c) => c.status === filter)

    const counts: Record<string, number> = {
        All:     containers.length,
        Running: containers.filter((c) => c.status === "Running").length,
        Stopped: containers.filter((c) => c.status === "Stopped").length,
        Crashed: containers.filter((c) => c.status === "Crashed").length,
    }

    return (
        <div className="p-6 max-w-5xl mx-auto">

            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h1 className="text-lg font-semibold text-gray-900 dark:text-white">{content.title}</h1>
                    <p className="text-sm text-gray-400 dark:text-gray-500 mt-0.5">{containers.length} {content.total}</p>
                </div>
                <button
                    onClick={() => setShowForm((v) => !v)}
                    className="flex items-center gap-1.5 text-sm px-3.5 py-1.5 rounded-lg bg-gray-900 dark:bg-white text-white dark:text-gray-900 font-medium hover:opacity-80 transition-opacity"
                >
                    {showForm ? content.cancel : content.newContainer}
                </button>
            </div>

            {/* Creation form */}
            {showForm && (
                <div className="mb-6 p-5 rounded-xl border border-gray-200 dark:border-white/6 bg-white dark:bg-white/3 flex flex-col gap-4">
                    <div className="grid grid-cols-2 gap-4">
                        <label className="flex flex-col gap-1.5">
                            <span className="text-xs font-medium text-gray-500 dark:text-gray-400">{content.form.name}</span>
                            <input
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                                placeholder={content.form.namePlaceholder}
                                className="text-sm px-3 py-2 rounded-lg border border-gray-200 dark:border-white/8 bg-gray-50 dark:bg-white/4 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-600 outline-none focus:border-blue-400 dark:focus:border-blue-500 transition-colors"
                            />
                        </label>
                        <label className="flex flex-col gap-1.5">
                            <span className="text-xs font-medium text-gray-500 dark:text-gray-400">{content.form.image}</span>
                            <input
                                value={image}
                                onChange={(e) => setImage(e.target.value)}
                                placeholder={content.form.imagePlaceholder}
                                className="text-sm px-3 py-2 rounded-lg border border-gray-200 dark:border-white/8 bg-gray-50 dark:bg-white/4 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-600 outline-none focus:border-blue-400 dark:focus:border-blue-500 transition-colors"
                            />
                        </label>
                    </div>
                    <label className="flex flex-col gap-1.5">
                        <span className="text-xs font-medium text-gray-500 dark:text-gray-400">
                            {content.form.args} <span className="font-normal text-gray-400">{content.form.argsHint}</span>
                        </span>
                        <input
                            value={args}
                            onChange={(e) => setArgs(e.target.value)}
                            placeholder={content.form.argsPlaceholder}
                            className="text-sm px-3 py-2 rounded-lg border border-gray-200 dark:border-white/8 bg-gray-50 dark:bg-white/4 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-600 outline-none focus:border-blue-400 dark:focus:border-blue-500 font-mono transition-colors"
                        />
                    </label>
                    <div className="flex items-center justify-between pt-1">
                        <label className="flex items-center gap-2 cursor-pointer select-none">
                            <input
                                type="checkbox"
                                checked={start}
                                onChange={(e) => setStart(e.target.checked)}
                                className="w-4 h-4 rounded accent-blue-500"
                            />
                            <span className="text-sm text-gray-600 dark:text-gray-300">{content.form.startImmediately}</span>
                        </label>
                        <button
                            onClick={handleCreate}
                            disabled={!name || !image}
                            className="text-sm px-4 py-1.5 rounded-lg bg-gray-900 dark:bg-white text-white dark:text-gray-900 font-medium hover:opacity-80 transition-opacity disabled:opacity-30 disabled:cursor-not-allowed"
                        >
                            {content.form.create}
                        </button>
                    </div>
                </div>
            )}

            {/* Filters */}
            <div className="flex gap-1 mb-5">
                {filters.map((f) => (
                    <button
                        key={f}
                        onClick={() => setFilter(f)}
                        className={`flex items-center gap-1.5 text-sm px-3 py-1 rounded-lg transition-colors ${
                            filter === f
                                ? "bg-gray-900 dark:bg-white text-white dark:text-gray-900 font-medium"
                                : "text-gray-500 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
                        }`}
                    >
                        {f}
                        <span className={`text-xs tabular-nums ${filter === f ? "opacity-60" : "opacity-40"}`}>
                            {counts[f]}
                        </span>
                    </button>
                ))}
            </div>

            {/* States */}
            {loading && <p className="text-sm text-gray-400 dark:text-gray-500">{content.loading}</p>}
            {error   && <p className="text-sm text-red-500">{error}</p>}
            {!loading && !error && filtered.length === 0 && (
                <p className="text-sm text-gray-400 dark:text-gray-500">{content.noContainers}</p>
            )}

            {/* List */}
            <div className="flex flex-col gap-2">
                {filtered.map((c) => (
                    <ContainerCard key={c.id} container={c} onDelete={remove} />
                ))}
            </div>
        </div>
    )
}
