import { useState, useEffect, useCallback } from "react"
import type { ContainerMetrics } from "@/lib/types"
import { fetchContainerMetrics } from "@/lib/api"

export function useContainerMetrics(id: string) {
    const [containerMetrics, setContainerMetrics] = useState<ContainerMetrics | null>(null)
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)

    const load = useCallback(async () => {
        try {
            setLoading(true)

            const data = await fetchContainerMetrics(id)

            setContainerMetrics(data)

            setError(null)
        } catch (err) {
            setError(err instanceof Error ? err.message : "Erreur inconnue")
        } finally {
            setLoading(false)
        }
    }, [id])

    useEffect(() => {
        const init = async () => {
            await load()
        }
        init()
        const interval = setInterval(() => load(), 5000)
        return () => clearInterval(interval)
    }, [load])
    return { containerMetrics, loading, error }
}