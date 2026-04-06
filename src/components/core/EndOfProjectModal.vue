<script setup lang="ts">
import { ref } from "vue";

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{
    (event: "update:show", value: boolean): void;
}>();

const showEnglish = ref(false);

const close = () => {
    emit("update:show", false);
};

const setEnglish = () => {
    showEnglish.value = true;
};

const setRussian = () => {
    showEnglish.value = false;
};
</script>

<template>
    <div
        v-if="props.show"
        class="fixed inset-0 z-9999 flex items-center justify-center bg-base-200/90 p-6 text-base-content"
        @click.self="close"
    >
        <div
            class="max-w-5xl w-full max-h-[90vh] overflow-y-auto rounded-3xl border border-base-content/10 bg-base-100/95 p-8 shadow-2xl backdrop-blur-sm"
        >
            <div class="flex flex-col gap-6">
                <div
                    class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
                >
                    <div
                        class="font-black tracking-[0.35em] text-center uppercase leading-tight"
                    >
                        &lt;&lt; || THE END. || &gt;&gt;
                    </div>
                    <div class="flex items-center gap-2 ml-auto">
                        <button
                            type="button"
                            class="btn btn-outline btn-sm"
                            :class="{ 'btn-primary': !showEnglish }"
                            @click="setRussian"
                        >
                            RU
                        </button>
                        <button
                            type="button"
                            class="btn btn-outline btn-sm"
                            :class="{ 'btn-primary': showEnglish }"
                            @click="setEnglish"
                        >
                            EN
                        </button>
                        <button
                            type="button"
                            class="btn btn-primary btn-sm"
                            @click="close"
                        >
                            Close
                        </button>
                    </div>
                </div>

                <div
                    class="space-y-6 text-sm sm:text-base leading-7 text-slate-200"
                >
                    <template v-if="showEnglish">
                        <p>
                            well, that's it. it's time to put a bold full stop
                            in the loader.
                            <br />i have made the final decision to completely
                            close the project and not return to it.
                        </p>

                        <p>---</p>

                        <p>
                            to be honest: i will no longer support it
                            financially or morally. the enthusiasm is dead, the
                            motivation to carry this whole thing has evaporated.
                            <br />i feel like i did everything i could and even
                            more. at one point we really overtook all similar
                            loaders and became the best at what we do (except
                            for newlauncher, of course).
                        </p>

                        <p>
                            for me it was an enormous experience, i learned a
                            lot in terms of code, made acquaintances, and
                            generally understood how all of this works from the
                            inside.
                        </p>

                        <p>
                            a huge thank you to everyone who supported the
                            project, even if you just used it.
                        </p>

                        <p>
                            respect to all users, admins and contributors who
                            were here from the very beginning until now. i
                            cannot put into words how grateful i am to you, you
                            are the best, i will always remember this.
                        </p>

                        <p>---</p>

                        <p>special thanks:</p>

                        <p>
                            @Arsenii754 - DevOps: truly one of the most
                            important people. he carried the entire
                            infrastructure himself, and if it weren't for his
                            help, we would have collapsed God knows when. tell
                            him thanks, he's a legend.
                        </p>

                        <p>
                            @Vakumok - Admin: one of the oldest members of the
                            team. he instantly closed tickets and managed
                            discord with the irc chat when chaos started there.
                        </p>

                        <p>
                            @W1xced - Admin/QA: always available, kept order,
                            and found so many critical bugs that without him the
                            loader would have fallen apart. thanks to him,
                            everything worked stably.
                        </p>

                        <p>---</p>

                        <p>
                            all source code is available in
                            <a
                                href="https://github.com/dest4590/CollapseLoader"
                                target="_blank"
                                rel="noreferrer noopener"
                                class="text-primary underline"
                            >
                                our organization
                            </a>
                            and in
                            <a
                                href="https://github.com/dest4590"
                                target="_blank"
                                rel="noreferrer noopener"
                                class="text-primary underline"
                            >
                                my account
                            </a>
                            , so if someone wants to continue this work, please
                            do so. i don't mind, but i cannot guarantee any
                            support or help, so you'll have to figure it out
                            yourself.
                        </p>

                        <p>the project is officially archived.</p>

                        <p>
                            but i moved all files to CDN, so you can still play
                            with the loader, but there will be no support or
                            updates.
                        </p>

                        <p>---</p>

                        <p>R.I.P CollapseLoader (2024 - 2026)</p>

                        <p>
                            it turned out to be a pretty good story.
                            <br />good luck to you all, and take care.
                        </p>

                        <p class="text-right font-semibold text-slate-300">
                            &gt; Purpl3 (aka пурпл, @dest4590)
                        </p>
                    </template>
                    <template v-else>
                        <p>
                            ну вот и всё. пришло время поставить жирную точку в
                            лоадере
                            <br />я принял окончательное решение полностью
                            закрыть проект и больше к нему не возвращаться
                        </p>

                        <p>---</p>

                        <p>
                            скажу как есть: я больше не вывожу его ни финансово,
                            ни морально. энтузиазм сдох, мотивация тащить это
                            всё испарилась
                            <br />я чувствую, что сделал всё, что мог, и даже
                            больше, в какой-то момент мы реально обогнали все
                            похожие лоадеры и стали лучшими в своем деле (кроме
                            newlauncher-а конечно)
                        </p>

                        <p>
                            для меня это был огромный опыт, я многому научился в
                            плане кода, оброс знакомствами и вообще понял, как
                            это всё работает извнутри
                        </p>

                        <p>
                            огромное спасибо всем, кто поддерживал проект, даже
                            если вы просто его юзали
                        </p>

                        <p>
                            респект всем юзерам, админам и контрибуторам,
                            которые были рядом с самого начала и до этого
                            момента, я не могу словами передать, как я вам
                            благодарен, вы лучшие, я всегда буду помнить это
                        </p>

                        <p>---</p>

                        <p>отдельное спасибо:</p>

                        <p>
                            @Arsenii754 - DevOps: реально один из важнейших
                            людей. он тащил на себе всю инфраструктуру, и если
                            бы не его помощь, мы бы накрылись ещё хрен знает
                            когда. скажите ему спасибо, он легенда.
                        </p>

                        <p>
                            @Vakumok - Admin: один из самых старых в команде.
                            моментально закрывал тикеты и держал дискорд с irc
                            чатом, когда там начинался хаос.
                        </p>

                        <p>
                            @W1xced - Admin/QA: всегда был на связи, следил за
                            порядком и нашел такое количество критических багов,
                            что без него лоадер бы рассыпался на куски.
                            благодаря ему всё работало стабильно.
                        </p>

                        <p>---</p>

                        <p>
                            все исходники есть в
                            <a
                                href="https://github.com/dest4590/CollapseLoader"
                                target="_blank"
                                rel="noreferrer noopener"
                                class="text-primary underline"
                            >
                                нашей организации
                            </a>
                            и в
                            <a
                                href="https://github.com/dest4590"
                                target="_blank"
                                rel="noreferrer noopener"
                                class="text-primary underline"
                            >
                                моём аккаунте
                            </a>
                            , так что если кто-то захочет продолжить это дело,
                            то пожалуйста, делайте это, я не против, но я не
                            могу гарантировать никакой поддержки и помощи, так
                            что придётся разбираться самим
                        </p>

                        <p>проект официально в архиве</p>

                        <p>
                            но я перенёс все файлы CDN, так что вы всё ещё
                            можете играть с лоадером, но поддержки и обновлений
                            не будет
                        </p>

                        <p>---</p>

                        <p>R.I.P CollapseLoader (2024 - 2026)</p>

                        <p>
                            неплохая получилась история
                            <br />удачи вам всем, и берегите себя.
                        </p>

                        <p class="text-right font-semibold text-slate-300">
                            &gt; Purpl3 (aka пурпл, @dest4590)
                        </p>
                    </template>
                </div>

                <button
                    type="button"
                    class="btn btn-primary btn-sm"
                    @click="close"
                >
                    Close
                </button>
            </div>
        </div>
    </div>
</template>
