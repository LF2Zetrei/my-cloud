import { useState, useEffect, useCallback } from "react"
import type { StorageFile } from "@/lib/types"
import { fetchFiles, uploadFile, deleteFile } from "@/lib/api"

export function useStorage() {

    const [files, setFiles] = useState<StorageFile[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)

    // fetch la liste
    const load = useCallback(async () => {
        try {
            setLoading(true)

            const data = await fetchFiles()

            setFiles (data)

            setError(null)
        } catch (err) {
            setError(err instanceof Error ? err.message : "Erreur inconnue")
        } finally {
            setLoading(false)
        }
    }, [])

    // refresh automatique toutes les 5s
    useEffect(() => {
        const init = async () => {
            await load()
        }
        init()
        const interval = setInterval(load, 5000)
        return () => clearInterval(interval)
    }, [load])

    // upload une file
    const upload = async (key: string, content: string) => {
        try {
            setLoading(true)

            await uploadFile(key, content)

            setError(null)
            await load()
        } catch (err) {
            setError(err instanceof Error ? err.message : "Erreur inconnue")
        } finally {
            setLoading(false)
        }
    }

    // supprimer une file
    const remove = async (key: string) => {
        try {
            setLoading(true)

            await deleteFile(key)

            setError(null)
            await load()
        } catch (err) {
            setError(err instanceof Error ? err.message : "Erreur inconnue")
        } finally {
            setLoading(false)
        }
    }

    return { files, loading, error, upload, remove, refetch: load }
}