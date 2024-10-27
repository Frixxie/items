import type { Handlers, PageProps } from "$fresh/server.ts";

interface Item {
    id: number;
    name: string;
    description: string;
    date_origin: Date;
}

export const handler: Handlers<Item[]> = {
    async GET(_req, ctx) {
        const items: Item[] = await fetch("http://localhost:3000/api/items")
            .then((res) => res.json());
        return ctx.render(items);
    },
};

export default function Items(props: PageProps<Item[]>) {
    return (
        <div>
            <h1>Items</h1>
            <ul>
                {props.data.map((item) => (
                    <li key={item.id}>
                        <h2>{item.name}</h2>
                        <p>{item.description}</p>
                        <p>{item.date_origin}</p>
                    </li>
                ))}
            </ul>
        </div>
    );
}
