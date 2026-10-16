"use client"

import { useState } from "react"
import { useStorage } from "@/hooks/useStorage"

const content = {
    title: "Storage",
    files: "files",
    uploadFile: "+ Upload file",
    cancel: "Cancel",
    form: {
        filename: "Filename",
        filenamePlaceholder: "my-file.txt",
        fileContent: "Content",
        fileContentPlaceholder: "File content...",
        upload: "Upload",
    },
    loading: "Loading...",
    noFiles: "No files found.",
    delete: "Delete",
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

export default function StoragePage() {
    const { files, loading, error, upload, remove } = useStorage()
    const [showForm, setShowForm]       = useState(false)
    const [key, setKey]                 = useState("")
    const [fileContent, setFileContent] = useState("")

    const handleUpload = async () => {
        await upload(key, fileContent)
        setKey(""); setFileContent(""); setShowForm(false)
    }

    return (
        <div className="p-6 max-w-5xl mx-auto">

            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h1 className="text-lg font-semibold text-gray-900 dark:text-white">{content.title}</h1>
                    <p className="text-sm text-gray-400 dark:text-gray-500 mt-0.5">{files.length} {content.files}</p>
                </div>
                <button
                    onClick={() => setShowForm((v) => !v)}
                    className="flex items-center gap-1.5 text-sm px-3.5 py-1.5 rounded-lg bg-gray-900 dark:bg-white text-white dark:text-gray-900 font-medium hover:opacity-80 transition-opacity"
                >
                    {showForm ? content.cancel : content.uploadFile}
                </button>
            </div>

            {/* Upload form */}
            {showForm && (
                <div className="mb-6 p-5 rounded-xl border border-gray-200 dark:border-white/6 bg-white dark:bg-white/3 flex flex-col gap-4">
                    <label className="flex flex-col gap-1.5">
                        <span className="text-xs font-medium text-gray-500 dark:text-gray-400">{content.form.filename}</span>
                        <input
                            value={key}
                            onChange={(e) => setKey(e.target.value)}
                            placeholder={content.form.filenamePlaceholder}
                            className="text-sm px-3 py-2 rounded-lg border border-gray-200 dark:border-white/8 bg-gray-50 dark:bg-white/4 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-600 outline-none focus:border-blue-400 dark:focus:border-blue-500 transition-colors"
                        />
                    </label>
                    <label className="flex flex-col gap-1.5">
                        <span className="text-xs font-medium text-gray-500 dark:text-gray-400">{content.form.fileContent}</span>
                        <textarea
                            value={fileContent}
                            onChange={(e) => setFileContent(e.target.value)}
                            rows={6}
                            placeholder={content.form.fileContentPlaceholder}
                            className="text-sm px-3 py-2 rounded-lg border border-gray-200 dark:border-white/8 bg-gray-50 dark:bg-white/4 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-600 outline-none focus:border-blue-400 dark:focus:border-blue-500 font-mono resize-y transition-colors"
                        />
                    </label>
                    <div className="flex justify-end">
                        <button
                            onClick={handleUpload}
                            disabled={!key}
                            className="text-sm px-4 py-1.5 rounded-lg bg-gray-900 dark:bg-white text-white dark:text-gray-900 font-medium hover:opacity-80 transition-opacity disabled:opacity-30 disabled:cursor-not-allowed"
                        >
                            {content.form.upload}
                        </button>
                    </div>
                </div>
            )}

            {/* States */}
            {loading && <p className="text-sm text-gray-400 dark:text-gray-500 mb-4">{content.loading}</p>}
            {error   && <p className="text-sm text-red-500 mb-4">{error}</p>}
            {!loading && !error && files.length === 0 && (
                <p className="text-sm text-gray-400 dark:text-gray-500">{content.noFiles}</p>
            )}

            {/* Grid */}
            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
                {files.map((file) => (
                    <div
                        key={file}
                        className="group flex flex-col gap-2 p-3.5 rounded-xl border border-gray-200 dark:border-white/6 bg-white dark:bg-white/3 hover:border-gray-300 dark:hover:border-white/12 transition-colors"
                    >
                        <span className="text-xl">{fileIcon(file)}</span>
                        <p className="text-xs font-medium text-gray-700 dark:text-gray-300 break-all leading-tight">
                            {file}
                        </p>
                        <button
                            onClick={() => remove(file)}
                            className="mt-auto text-xs self-start text-gray-400 dark:text-gray-600 hover:text-red-500 dark:hover:text-red-400 transition-colors opacity-0 group-hover:opacity-100"
                        >
                            {content.delete}
                        </button>
                    </div>
                ))}
            </div>
        </div>
    )
}
