import {
    Button,
    Dropdown,
    DropdownItem,
    DropdownMenu,
    DropdownTrigger,
    Input,
    Switch,
    Textarea,
} from '@nextui-org/react';
import toast, { Toaster } from 'react-hot-toast';
import { useTranslation } from 'react-i18next';
import React from 'react';

import { useConfig } from '../../../hooks/useConfig';
import { useToastStyle } from '../../../hooks';
import { INSTANCE_NAME_CONFIG_KEY } from '../../../utils/service_instance';

export const providerPresets = {
    openai: {
        requestPath: 'https://api.openai.com/v1/chat/completions',
        model: 'gpt-4o-mini',
    },
    openrouter: {
        requestPath: 'https://openrouter.ai/api/v1/chat/completions',
        model: 'openai/gpt-4o-mini',
    },
    siliconflow: {
        requestPath: 'https://api.siliconflow.cn/v1/chat/completions',
        model: '',
    },
    dashscope_cn: {
        requestPath: 'https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions',
        model: 'qwen-vl-plus',
    },
    custom: {
        requestPath: '',
        model: '',
    },
};

export const defaultPrompt =
    'Read the image directly and answer according to the visible content. If the image contains text, avoid OCR-style transcription errors by reasoning from the image itself. Provide a helpful response for translation, explanation, language learning, word-by-word notes, or grammar analysis as appropriate.';

export const defaultRequestArguments = JSON.stringify(
    {
        temperature: 0.1,
    },
    null,
    4
);

export const defaultExtraHeaders = JSON.stringify({}, null, 4);

function createDefaultConfig(t) {
    return {
        [INSTANCE_NAME_CONFIG_KEY]: t('services.vision.openai_compatible.title'),
        enable: true,
        provider: 'openai',
        requestPath: providerPresets.openai.requestPath,
        model: providerPresets.openai.model,
        apiKey: '',
        authType: 'bearer',
        stream: false,
        imageDetail: 'auto',
        prompt: defaultPrompt,
        requestArguments: defaultRequestArguments,
        extraHeaders: defaultExtraHeaders,
    };
}

function validateJsonObject(value, label) {
    const parsed = JSON.parse(value);
    if (parsed === null || Array.isArray(parsed) || typeof parsed !== 'object') {
        throw new Error(`${label} must be a JSON object`);
    }
}

export function Config(props) {
    const { instanceKey, updateServiceList, onClose } = props;
    const { t } = useTranslation();
    const toastStyle = useToastStyle();
    const defaultConfig = createDefaultConfig(t);
    const [visionConfig, setVisionConfig] = useConfig(instanceKey, defaultConfig, { sync: false });
    const effectiveVisionConfig = visionConfig === null ? null : { ...defaultConfig, ...visionConfig };

    return (
        effectiveVisionConfig !== null && (
            <form
                onSubmit={(e) => {
                    e.preventDefault();
                    try {
                        validateJsonObject(
                            effectiveVisionConfig.requestArguments || defaultRequestArguments,
                            'Request arguments'
                        );
                        validateJsonObject(effectiveVisionConfig.extraHeaders || defaultExtraHeaders, 'Extra headers');
                        setVisionConfig(effectiveVisionConfig, true);
                        updateServiceList(instanceKey);
                        onClose();
                    } catch (error) {
                        toast.error(error.toString(), { style: toastStyle });
                    }
                }}
            >
                <Toaster />
                <div className='config-item'>
                    <Input
                        label={t('services.instance_name')}
                        labelPlacement='outside-left'
                        value={effectiveVisionConfig[INSTANCE_NAME_CONFIG_KEY]}
                        variant='bordered'
                        classNames={{
                            base: 'justify-between',
                            label: 'text-[length:--nextui-font-size-medium]',
                            mainWrapper: 'max-w-[50%]',
                        }}
                        onValueChange={(value) => {
                            setVisionConfig({
                                ...effectiveVisionConfig,
                                [INSTANCE_NAME_CONFIG_KEY]: value,
                            });
                        }}
                    />
                </div>
                <div className='config-item'>
                    <h3 className='my-auto'>{t('services.vision.openai_compatible.provider')}</h3>
                    <Dropdown>
                        <DropdownTrigger>
                            <Button variant='bordered'>
                                {t(`services.vision.openai_compatible.providers.${effectiveVisionConfig.provider}`)}
                            </Button>
                        </DropdownTrigger>
                        <DropdownMenu
                            aria-label='vision provider'
                            onAction={(key) => {
                                const preset = providerPresets[key];
                                setVisionConfig({
                                    ...effectiveVisionConfig,
                                    provider: key,
                                    requestPath: preset.requestPath,
                                    model: preset.model,
                                });
                            }}
                        >
                            {Object.keys(providerPresets).map((provider) => (
                                <DropdownItem key={provider}>
                                    {t(`services.vision.openai_compatible.providers.${provider}`)}
                                </DropdownItem>
                            ))}
                        </DropdownMenu>
                    </Dropdown>
                </div>
                <div className='config-item'>
                    <Input
                        label={t('services.vision.openai_compatible.request_path')}
                        labelPlacement='outside-left'
                        value={effectiveVisionConfig.requestPath}
                        variant='bordered'
                        classNames={{
                            base: 'justify-between',
                            label: 'text-[length:--nextui-font-size-medium]',
                            mainWrapper: 'max-w-[50%]',
                        }}
                        onValueChange={(value) => {
                            setVisionConfig({
                                ...effectiveVisionConfig,
                                requestPath: value,
                            });
                        }}
                    />
                </div>
                <div className='config-item'>
                    <Input
                        label={t('services.vision.openai_compatible.model')}
                        labelPlacement='outside-left'
                        value={effectiveVisionConfig.model}
                        variant='bordered'
                        classNames={{
                            base: 'justify-between',
                            label: 'text-[length:--nextui-font-size-medium]',
                            mainWrapper: 'max-w-[50%]',
                        }}
                        onValueChange={(value) => {
                            setVisionConfig({
                                ...effectiveVisionConfig,
                                model: value,
                            });
                        }}
                    />
                </div>
                <div className='config-item'>
                    <Input
                        label={t('services.vision.openai_compatible.api_key')}
                        labelPlacement='outside-left'
                        type='password'
                        value={effectiveVisionConfig.apiKey}
                        variant='bordered'
                        classNames={{
                            base: 'justify-between',
                            label: 'text-[length:--nextui-font-size-medium]',
                            mainWrapper: 'max-w-[50%]',
                        }}
                        onValueChange={(value) => {
                            setVisionConfig({
                                ...effectiveVisionConfig,
                                apiKey: value,
                            });
                        }}
                    />
                </div>
                <div className='config-item'>
                    <h3 className='my-auto'>{t('services.vision.openai_compatible.auth_type')}</h3>
                    <Dropdown>
                        <DropdownTrigger>
                            <Button variant='bordered'>
                                {t(`services.vision.openai_compatible.auth.${effectiveVisionConfig.authType}`)}
                            </Button>
                        </DropdownTrigger>
                        <DropdownMenu
                            aria-label='auth type'
                            onAction={(key) => {
                                setVisionConfig({
                                    ...effectiveVisionConfig,
                                    authType: key,
                                });
                            }}
                        >
                            <DropdownItem key='bearer'>{t('services.vision.openai_compatible.auth.bearer')}</DropdownItem>
                            <DropdownItem key='api_key'>{t('services.vision.openai_compatible.auth.api_key')}</DropdownItem>
                        </DropdownMenu>
                    </Dropdown>
                </div>
                <div className='config-item'>
                    <h3 className='my-auto'>{t('services.vision.openai_compatible.image_detail')}</h3>
                    <Dropdown>
                        <DropdownTrigger>
                            <Button variant='bordered'>
                                {t(`services.vision.openai_compatible.detail.${effectiveVisionConfig.imageDetail}`)}
                            </Button>
                        </DropdownTrigger>
                        <DropdownMenu
                            aria-label='image detail'
                            onAction={(key) => {
                                setVisionConfig({
                                    ...effectiveVisionConfig,
                                    imageDetail: key,
                                });
                            }}
                        >
                            <DropdownItem key='auto'>{t('services.vision.openai_compatible.detail.auto')}</DropdownItem>
                            <DropdownItem key='low'>{t('services.vision.openai_compatible.detail.low')}</DropdownItem>
                            <DropdownItem key='high'>{t('services.vision.openai_compatible.detail.high')}</DropdownItem>
                        </DropdownMenu>
                    </Dropdown>
                </div>
                <div className='config-item'>
                    <Switch
                        isSelected={effectiveVisionConfig.stream}
                        onValueChange={(value) => {
                            setVisionConfig({
                                ...effectiveVisionConfig,
                                stream: value,
                            });
                        }}
                        classNames={{
                            base: 'flex flex-row-reverse justify-between w-full max-w-full',
                        }}
                    >
                        {t('services.vision.openai_compatible.stream')}
                    </Switch>
                </div>
                <div className='config-item'>
                    <Textarea
                        label={t('services.vision.openai_compatible.prompt')}
                        labelPlacement='outside'
                        variant='faded'
                        value={effectiveVisionConfig.prompt}
                        onValueChange={(value) => {
                            setVisionConfig({
                                ...effectiveVisionConfig,
                                prompt: value,
                            });
                        }}
                    />
                </div>
                <div className='config-item'>
                    <Textarea
                        label={t('services.vision.openai_compatible.request_arguments')}
                        labelPlacement='outside'
                        variant='faded'
                        value={effectiveVisionConfig.requestArguments}
                        onValueChange={(value) => {
                            setVisionConfig({
                                ...effectiveVisionConfig,
                                requestArguments: value,
                            });
                        }}
                    />
                </div>
                <div className='config-item'>
                    <Textarea
                        label={t('services.vision.openai_compatible.extra_headers')}
                        labelPlacement='outside'
                        variant='faded'
                        value={effectiveVisionConfig.extraHeaders}
                        onValueChange={(value) => {
                            setVisionConfig({
                                ...effectiveVisionConfig,
                                extraHeaders: value,
                            });
                        }}
                    />
                </div>
                <Button
                    type='submit'
                    fullWidth
                    color='primary'
                >
                    {t('common.save')}
                </Button>
            </form>
        )
    );
}
