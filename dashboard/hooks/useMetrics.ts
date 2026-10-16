import { useState, useEffect, useCallback } from "react"
import type { Metrics } from "@/lib/types"
import { fetchMetrics } from "@/lib/api"

export function useMetrics() {
    const [metrics, setMetrics] = useState<Metrics | null>(null)
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)

    const load = useCallback(async () => {
        try {
            setLoading(true)

            const data = await fetchMetrics()

            setMetrics(data)

            setError(null)
        } catch (err) {
            setError(err instanceof Error ? err.message : "Erreur inconnue")
        } finally {
            setLoading(false)
        }
    }, [])

    useEffect(() => {
        const init = async () => {
            await load()
        }
        init()
        const interval = setInterval(load, 5000)
        return () => clearInterval(interval)
    }, [load])
    return { metrics, loading, error }
}