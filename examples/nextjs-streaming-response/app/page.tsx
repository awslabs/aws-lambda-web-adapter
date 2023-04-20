import { getBaseUrl } from "@/lib/getBaseUrl";
import { Ping } from "@/ui/ping";
import { Suspense } from "react";
import { RecommendedProducts, RecommendedProductsSkeleton } from "./_components/recommanded-products";
import { Reviews, ReviewsSkeleton } from "./_components/reviews";
import { SingleProduct } from "./_components/single-product";

export default async function Page({ params }: { params: { id: string } }) {
  return (
    <div className="space-y-8 lg:space-y-14">
      {/* @ts-expect-error Async Server Component */}
      <SingleProduct data={fetch(
        `${getBaseUrl()}/api/products?id=1`,
        { cache: 'no-store' }
      )} />

      <div className="relative">
        <div className="absolute top-2 -left-4">
          <Ping />
        </div>
      </div>

      <Suspense fallback={<RecommendedProductsSkeleton />}>
        {/* @ts-expect-error Async Server Component */}
        <RecommendedProducts
          path=""
          data={fetch(
            `${getBaseUrl()}/api/products?delay=5000&filter=1`,
            {
              cache: 'no-store'
            }
          )}
        />
      </Suspense>

      <div className="relative">
        <div className="absolute top-2 -left-4">
          <Ping />
        </div>
      </div>

      <Suspense fallback={<ReviewsSkeleton />}>
        {/* @ts-expect-error Async Server Component */}
        <Reviews
          data={fetch(
            `${getBaseUrl()}/api/reviews?delay=10000`,
            { cache: 'no-store' }
          )}
        />

      </Suspense>
    </div>
  )
}
