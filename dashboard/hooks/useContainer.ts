import { useState, useEffect, useCallback } from "react"
import type { Container } from "@/lib/types"
import { fetchContainers, createContainer, deleteContainer } from "@/lib/api"

export function useContainers() {

    const [containers, setContainers] = useState<Container[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)

    // fetch la liste
    const load = useCallback(async () => {
        try {
            setLoading(true)

            const data = await fetchContainers()

            setContainers(data)

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

    // créer un container
    const create = async (name: string, image: string, args: string[], start: boolean) => {
        try {
            setLoading(true)

            await createContainer(name, image, args, start)

            setError(null)
            await load()
        } catch (err) {
            setError(err instanceof Error ? err.message : "Erreur inconnue")
        } finally {
            setLoading(false)
        }
    }

    // supprimer un container
    const remove = async (id: string) => {
        try {
            setLoading(true)

            await deleteContainer(id)

            setError(null)
            await load()
        } catch (err) {
            setError(err instanceof Error ? err.message : "Erreur inconnue")
        } finally {
            setLoading(false)
        }
    }

    return { containers, loading, error, create, remove, refetch: load }
}